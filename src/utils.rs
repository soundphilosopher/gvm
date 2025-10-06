use futures_lite::stream::StreamExt;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    env,
    error::Error,
    io,
    path::{Path, PathBuf},
};

#[cfg(unix)]
use std::os::unix::fs as unix_fs;

use crate::{error, info, success, Res};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FilteredRelease {
    pub version: String,
    pub url: String,
}

/// Returns `true` if the version is stable. It strips the "go" prefix and
/// considers a version unstable if it contains "rc", "beta", or "alpha".
pub fn is_stable_version(version: &str) -> bool {
    let trimmed = version.strip_prefix("go").unwrap_or(version);
    !(trimmed.contains("rc") || trimmed.contains("beta") || trimmed.contains("alpha"))
}

/// Ensures that a given version string is prefixed with "go".
///
/// This function takes a version string and checks if it starts with "go".
/// If it doesn't, it prepends "go" to the version string.
///
/// # Parameters
///
/// * `version`: A String representing the version number, which may or may not start with "go".
///
/// # Returns
///
/// A String that always starts with "go", followed by the version number.
pub fn get_real_version(version: String) -> String {
    let real_version = if version.starts_with("go") {
        version
    } else {
        format!("go{}", version)
    };

    real_version
}

/// Parses a version string into its numeric base parts and an optional suffix.
/// For example:
///   - "go1.24.0"  => (vec![1, 24, 0], "")
///   - "go1.24rc1" => (vec![1, 24], "rc1")
fn parse_version_parts(version: &str) -> (Vec<u32>, String) {
    // Regex to capture the numeric part and the rest.
    // Captures: 1) the numeric part (e.g. "1.24" or "1.24.0") and 2) any trailing characters.
    let re = Regex::new(r"^go(\d+(?:\.\d+)*)(.*)$").unwrap();
    if let Some(caps) = re.captures(version) {
        let base = &caps[1];
        let suffix = &caps[2];
        let base_parts: Vec<u32> = base
            .split('.')
            .filter_map(|s| s.parse::<u32>().ok())
            .collect();
        return (base_parts, suffix.to_string());
    }
    (vec![], String::new())
}

/// Custom comparator for version strings.
/// 1. Compares the numeric parts.
/// 2. If the base versions are equal, then:
///    - If one version is unstable (non‑empty suffix) and the other is stable,
///      the unstable version comes first.
///    - If both are unstable, compare the suffixes lexicographically.
pub fn cmp_versions(a: &str, b: &str) -> Ordering {
    let (base_a, suffix_a) = parse_version_parts(a);
    let (base_b, suffix_b) = parse_version_parts(b);

    // First compare the numeric (base) versions.
    match base_a.cmp(&base_b) {
        Ordering::Equal => {
            // For the same base version, we want unstable versions (non‑empty suffix) first.
            match (suffix_a.is_empty(), suffix_b.is_empty()) {
                (false, true) => Ordering::Less,    // a is unstable, b is stable
                (true, false) => Ordering::Greater, // a is stable, b is unstable
                (true, true) => Ordering::Equal,    // both are stable
                (false, false) => suffix_a.cmp(&suffix_b), // both unstable: sort lexically
            }
        }
        ord => ord,
    }
}

/// Attempts to retrieve the user's home directory.
///
/// This function tries to get the value of the "HOME" environment variable,
/// which typically represents the user's home directory on Unix-like systems.
///
/// # Returns
///
/// - `Some(String)` containing the path to the user's home directory if the
///   "HOME" environment variable is set and valid.
/// - `None` if the "HOME" environment variable is not set or cannot be read.
fn get_home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| error!("Cannot access HOME dir"))
}

/// Retrieves the path of the current shell.
///
/// This function retrieves the value of the "SHELL" environment variable,
/// which represents the path to the currently active shell.
///
/// # Returns
///
/// * `Some(String)`: If the "SHELL" environment variable is set, the function
///   returns `Some` containing the path to the shell as a string.
/// * `None`: If the "SHELL" environment variable is not set or is empty,
///   the function returns `None`.
fn get_shell() -> Option<String> {
    env::var("SHELL").ok()
}

/// Determines the path to the shell configuration file based on the current shell.
///
/// This function attempts to identify the user's shell (bash or zsh) and returns
/// the path to the appropriate configuration file (.bashrc or .zshrc).
///
/// # Returns
///
/// A `String` containing the path to the shell configuration file.
///
/// # Panics
///
/// This function will panic with an error message in the following cases:
/// - If the shell is neither bash nor zsh.
/// - If the SHELL environment variable cannot be retrieved.
/// - If the home directory cannot be determined for the identified shell.
pub fn get_shell_config_file_path() -> Result<PathBuf, String> {
    match get_shell() {
        Some(shell_path) => {
            if shell_path.ends_with("/bash") {
                let home = get_home_dir();
                return Ok(home.join(".bashrc"));
            } else if shell_path.ends_with("/zsh") {
                let home = get_home_dir();
                return Ok(home.join(".zshrc"));
            } else {
                return Err(format!("Unsupported shell: {}", shell_path));
            }
        }
        None => {
            return Err("Failed to retrieve SHELL environment variable".to_string());
        }
    }
}

/// Returns the base file path for the GVM (Go Version Manager) system.
///
/// This function determines the location of the base directory used by GVM.
/// It first attempts to use the user's home directory. If available, it appends
/// the GVM-specific path. If the home directory cannot be determined,
/// it falls back to a temporary directory.
///
/// # Returns
///
/// A `String` representing the full path to the GVM base directory:
/// - `~/.gvm` if the home directory is available
/// - `/tmp/gvm` as a fallback if the home directory cannot be determined
pub fn get_gvm_base_file_path() -> PathBuf {
    let home = get_home_dir();
    home.join(".gvm")
}

/// Returns the path to the cache directory for the GVM (Go Version Manager) system.
///
/// This function determines the location of the cache directory used by GVM.
/// It first attempts to use the user's home directory. If available, it appends
/// the GVM-specific cache path. If the home directory cannot be determined,
/// it falls back to a temporary directory.
///
/// # Returns
///
/// A `String` representing the full path to the cache directory:
/// - `~/.gvm/cache` if the home directory is available
/// - `/tmp/gvm/cache` as a fallback if the home directory cannot be determined
pub fn get_cache_dir() -> PathBuf {
    let gvm_path = get_gvm_base_file_path();
    gvm_path.join("cache")
}

/// Returns the file path for the environment configuration used by GVM (Go Version Manager).
///
/// This function determines the location of the environment file used by the GVM system.
/// It first attempts to use the user's home directory. If available, it appends the GVM-specific path.
/// If the home directory cannot be determined, it falls back to a temporary directory.
///
/// # Returns
///
/// A `String` representing the full path to the environment file:
/// - `~/.gvm/environment` if the home directory is available
/// - `/tmp/gvm/environment` as a fallback if the home directory cannot be determined
pub fn get_environment_file_path() -> PathBuf {
    let gvm_path = get_gvm_base_file_path();
    gvm_path.join("environment")
}

/// Returns the file path for the version configuration.
///
/// This function determines the location of the version file used by the GVM (Go Version Manager) system.
/// It first attempts to use the user's home directory. If available, it appends the GVM-specific path.
/// If the home directory cannot be determined, it falls back to a temporary directory.
///
/// # Returns
///
/// A `String` representing the full path to the version file:
/// - `~/.gvm/version` if the home directory is available
/// - `/tmp/gvm/version` as a fallback if the home directory cannot be determined
pub fn get_version_file_path() -> PathBuf {
    let gvm_path = get_gvm_base_file_path();
    gvm_path.join("version")
}

/// Returns the file path for the package configuration.
///
/// This function determines the location of the package file used by the GVM (Go Version Manager) system.
/// It first attempts to use the user's home directory. If available, it appends the GVM-specific path.
/// If the home directory cannot be determined, it falls back to a temporary directory.
///
/// # Returns
///
/// A `String` representing the full path to the package file:
/// - `~/.gvm/package` if the home directory is available
/// - `/tmp/gvm/package` as a fallback if the home directory cannot be determined
pub fn get_package_file_path() -> PathBuf {
    let gvm_path = get_gvm_base_file_path();
    gvm_path.join("package")
}

/// Returns the file path for the archive configuration.
///
/// This function determines the location of the archive file used by the GVM (Go Version Manager) system.
/// It first attempts to use the user's home directory. If available, it appends the GVM-specific path.
/// If the home directory cannot be determined, it falls back to a temporary directory.
///
/// # Returns
///
/// A `String` representing the full path to the archive file:
/// - `~/.gvm/archive` if the home directory is available
/// - `/tmp/gvm/archive` as a fallback if the home directory cannot be determined
pub fn get_archive_file_path() -> PathBuf {
    let gvm_path = get_gvm_base_file_path();
    gvm_path.join("archive")
}

/// Returns the file path for the alias configuration.
///
/// This function determines the location of the alias file used by the GVM (Go Version Manager) system.
/// It first attempts to use the user's home directory. If available, it appends the GVM-specific path.
/// If the home directory cannot be determined, it falls back to a temporary directory.
///
/// # Returns
///
/// A `String` representing the full path to the alias file:
/// - `~/.gvm/alias` if the home directory is available
/// - `/tmp/gvm/alias` as a fallback if the home directory cannot be determined
pub fn get_alias_file_path() -> PathBuf {
    let gvm_path = get_gvm_base_file_path();
    gvm_path.join("alias")
}

/// Lists all installed Go versions managed by GVM.
///
/// This function scans the GVM version directory and collects the names of all
/// subdirectories, which are assumed to represent installed Go versions.
///
/// # Returns
///
/// Returns a `Result` containing:
/// - `Ok(Vec<String>)`: A vector of strings, where each string is the name of an
///   installed Go version.
/// - `Err(Box<dyn Error + Send + Sync>)`: An error if the version directory
///   cannot be read or if there are issues accessing the directory entries.
///
/// # Errors
///
/// This function will return an error if:
/// - The version directory path cannot be retrieved or accessed.
/// - There are issues reading the directory entries.
/// - The directory entry names cannot be converted to strings.
pub async fn list_installed_versions() -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
    let version_path = get_version_file_path();
    let mut versions = Vec::new();

    let mut entries = async_fs::read_dir(&version_path).await?;

    while let Some(entry) = entries.try_next().await? {
        if entry.file_type().await?.is_dir() {
            let version_name = entry.file_name().into_string().unwrap_or_default();
            versions.push(version_name);
        }
    }

    Ok(versions)
}

/// Lists all aliases defined in the GVM (Go Version Manager) system.
///
/// This function reads the alias directory and collects the names of all
/// files, which are assumed to represent defined aliases.
///
/// # Returns
///
/// Returns a `Result` containing:
/// - `Ok(Vec<String>)`: A vector of strings, where each string is the name of a
///   defined alias.
/// - `Err(Box<dyn Error + Send + Sync>)`: An error if the alias directory
///   cannot be read or if there are issues accessing the directory entries.
///
/// # Errors
///
/// This function will return an error if:
/// - The alias directory path cannot be retrieved or accessed.
/// - There are issues reading the directory entries.
/// - The directory entry names cannot be converted to strings.
pub async fn list_aliases() -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
    let alias_path = get_alias_file_path();
    let mut aliases = Vec::new();

    let mut entries = async_fs::read_dir(&alias_path).await?;

    while let Some(enty) = entries.try_next().await? {
        let alias_name = enty.file_name().into_string().unwrap_or_default();
        aliases.push(alias_name);
    }

    Ok(aliases)
}

/// Reads the cache file and returns all cached releases, applying filtering criteria,
/// and then sorts the list in ascending order (so that the latest version is at the bottom).
///
/// - `cache_file`: Path to the cached file.
/// - `version_filter`: Optional filter for the version string (e.g. "1.21.1" for exact match
///   or "1.21.*" for wildcard matching). If the provided filter does not start with "go", it will be prefixed.
/// - `stable_only`: When `true`, only releases with stable version strings are returned.
pub async fn list_cached_versions<P: AsRef<Path>>(
    cache_file: P,
    version_filter: Option<String>,
    stable_only: bool,
) -> Result<Vec<FilteredRelease>, Box<dyn Error + Send + Sync>> {
    // Read and deserialize the cached JSON file.
    let data = async_fs::read_to_string(&cache_file).await?;
    let mut releases: Vec<FilteredRelease> = serde_json::from_str(&data)?;

    // Ensure the version filter (if provided) starts with "go".
    let version_filter = version_filter.map(|f| {
        if f.starts_with("go") {
            f
        } else {
            format!("go{}", f)
        }
    });

    // Filter releases based on stability and version string.
    releases.retain(|r: &FilteredRelease| {
        if stable_only && !is_stable_version(&r.version) {
            return false;
        }
        if let Some(ref filter) = version_filter {
            if filter.ends_with('*') {
                let prefix = &filter[..filter.len() - 1];
                r.version.starts_with(prefix)
            } else {
                r.version == *filter
            }
        } else {
            true
        }
    });

    // Sort the filtered releases in ascending order using our custom comparator.
    releases.sort_by(|a, b| cmp_versions(&a.version, &b.version));

    Ok(releases)
}

/// Removes an existing symbolic link if it exists.
///
/// This function checks if the given path exists and is a symbolic link.
/// If so, it removes the symbolic link. This operation is only performed
/// on Unix-like systems.
///
/// # Parameters
///
/// * `link`: A path-like object representing the potential symbolic link to be removed.
///
/// # Returns
///
/// Returns `io::Result<()>`:
/// - `Ok(())` if the operation was successful (either the symlink was removed or didn't exist).
/// - `Err(e)` if an error occurred during the file system operations.
///
/// # Platform-specific behavior
///
/// The actual removal of the symlink is only performed on Unix-like systems.
pub async fn remove_existing_symlink<P: AsRef<Path>>(link: P) -> io::Result<()> {
    let link = link.as_ref();
    if link.exists() {
        // Use symlink_metadata to avoid following the symlink.
        let metadata = async_fs::symlink_metadata(link).await?;
        if metadata.file_type().is_symlink() {
            info!("Removing existing symlink: {}", link.display());
            #[cfg(unix)]
            {
                async_fs::remove_file(link).await?;
            }
        }
    }
    Ok(())
}

/// Creates a symbolic link.
///
/// This function creates a symbolic link pointing to the `original` path at the `link` location.
/// It is only available on Unix-like systems.
///
/// # Parameters
///
/// * `original`: A path-like object representing the target that the symlink will point to.
/// * `link`: A path-like object representing the location where the symlink will be created.
///
/// # Returns
///
/// Returns `io::Result<()>`:
/// - `Ok(())` if the symlink was successfully created.
/// - `Err(e)` if an error occurred during the creation of the symlink.
///
/// # Platform-specific behavior
///
/// This function is only available on Unix-like systems. On other platforms, it will not be compiled.
pub async fn create_symlink<P: AsRef<Path>, Q: AsRef<Path>>(
    original: P,
    link: Q,
) -> io::Result<()> {
    let link = link.as_ref();
    let original = original.as_ref();
    // Remove an existing symlink, if any.
    match remove_existing_symlink(link).await {
        Ok(()) => success!("Removed existing symlink: {}", link.display()),
        Err(e) => error!("Error removing existing symlink: {}", e),
    }

    #[cfg(unix)]
    {
        info!(
            "Creating symlink: {} -> {}",
            original.display(),
            link.display()
        );
        unix_fs::symlink(original, link)
    }
}

/// Activates a specified Go version in the GVM (Go Version Manager) system.
///
/// This function performs the following tasks:
/// 1. Verifies if the specified version exists.
/// 2. Sets the version as active by writing it to the active file.
/// 3. Creates a default alias for the active version.
///
/// # Parameters
///
/// * `version`: A String representing the Go version to activate. It can be with or without the "go" prefix.
///
/// # Returns
///
/// * `Ok(())` if the version is successfully activated.
/// * `Err(Box<dyn Error + Send + Sync>)` if an error occurs during the activation process,
///   such as the specified version not being found or issues with file operations.
///
/// # Errors
///
/// This function will return an error if:
/// * The specified version is not found in the GVM system.
/// * There are issues writing to the active file.
/// * There are problems creating the default alias symlink.
pub async fn activate_version(version: String) -> Result<(), Box<dyn Error + Send + Sync>> {
    let real_version = get_real_version(version);
    let version_path = get_version_file_path();
    let release_dir = version_path.join(&real_version);

    if !release_dir.is_dir() {
        error!(
            "Version '{}' not found. Use 'gvm list' to see available versions.",
            real_version
        );
    }

    info!("Activating version '{}' ...", real_version);
    let active_path = version_path.join("active");

    match async_fs::write(active_path, &real_version).await {
        Ok(_) => info!("Version '{}' activated.", real_version),
        Err(e) => error!("Error writing to active file: {}", e),
    }

    info!("Create default alias for version '{}' ...", real_version);
    let alias_path = get_alias_file_path();
    let alias_file_path = alias_path.join("default");
    match create_symlink(&release_dir, alias_file_path).await {
        Ok(()) => success!("Default alias for version '{}' created.", real_version),
        Err(e) => error!(
            "Error creating default alias for version '{}': {}",
            real_version, e
        ),
    }

    info!("Create build cache for version '{}' ...", real_version);
    let cache_dir = get_cache_dir();
    let version_build_cache_dir = cache_dir.join(&real_version).join("go-build");
    match async_fs::create_dir_all(&version_build_cache_dir).await {
        Ok(_) => success!("Build cache for version '{}' created.", real_version),
        Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists => {
            info!("Build cache for version '{}' already exists.", real_version)
        }
        Err(e) => error!(
            "Error creating build cache for version '{}': {}",
            real_version, e
        ),
    }

    info!("Create go package path for version '{}' ...", real_version);
    let package_path = get_package_file_path();
    let version_package_path = package_path.join(&real_version).join("bin");
    match async_fs::create_dir_all(&version_package_path).await {
        Ok(_) => success!("Go package path for version '{}' created.", real_version),
        Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists => {
            info!(
                "Go package path for version '{}' already exists.",
                real_version
            )
        }
        Err(e) => error!(
            "Error creating go package path for version '{}': {}",
            real_version, e
        ),
    }

    init_go_environment(Some(real_version.clone())).await?;

    success!(
        "Go version '{}' activated successfully. Please reload profile.",
        real_version
    );

    Ok(())
}

/// Initializes the Go environment for a specified version.
///
/// This function sets up the necessary environment variables for a given Go version,
/// including GOROOT, GOPATH, GOCACHE, and updates the PATH.
///
/// # Parameters
///
/// * `version`: An `Option<String>` representing the Go version to initialize.
///              If `Some`, it should contain the version string (e.g., "go1.16.5").
///              If `None`, an error message will be logged.
///
/// # Returns
///
/// * `Res<()>`: A Result type alias. Returns `Ok(())` if the environment is successfully
///              initialized, or an error if the initialization fails or no version is provided.
///
/// # Errors
///
/// This function will return an error if:
/// * No version is provided (i.e., `version` is `None`).
/// * There are issues setting environment variables or reading the current PATH.
pub async fn init_go_environment(version: Option<String>) -> Res<()> {
    let active_version = match version {
        Some(v) => v,
        None => match get_active_version().await {
            Some(v) => v,
            None => error!("No active version found. Use 'gvm list' to see available versions."),
        },
    };

    info!(
        "Prepare go environment for version '{}' ...",
        active_version
    );

    info!("Prepare environment for version {} ...", &active_version);
    let environment_path = get_environment_file_path();
    match async_fs::create_dir_all(&environment_path).await {
        Ok(_) => success!("Environment directory created."),
        Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists => {
            info!("Environment directory already exists.")
        }
        Err(e) => error!("Error creating environment directory: {}", e),
    }

    let environment_file_path = environment_path.join("go.env");
    let version_path = get_version_file_path();
    let cache_dir = get_cache_dir();
    let package_path = get_package_file_path();

    let goroot = version_path.join(&active_version);
    let gocache = cache_dir.join(&active_version).join("go-build");
    let gopath = package_path.join(&active_version);

    let env_vars = vec![
        ("GOROOT", goroot.to_string_lossy()),
        ("GOCACHE", gocache.to_string_lossy()),
        ("GOPATH", gopath.to_string_lossy()),
        ("GOENV", environment_file_path.to_string_lossy()),
    ];

    let mut env_content = String::new();

    for (env_key, env_value) in env_vars {
        if env_value.contains(' ') || env_value.contains('"') || env_value.contains('\'') {
            env_content.push_str(&format!(
                "{}=\"{}\"\n",
                env_key,
                env_value.replace('"', "\\\"")
            ));
        } else {
            env_content.push_str(&format!("{}={}\n", env_key, env_value));
        }
    }

    async_fs::write(&environment_file_path, env_content).await?;

    success!("Go environment prepared for version '{}'.", &active_version);

    Ok(())
}

/// Retrieves the currently active Go version managed by GVM.
///
/// This function reads the 'active' file in the GVM version directory
/// to determine which Go version is currently set as active.
///
/// # Returns
///
/// - `Some(String)`: The active Go version as a string (e.g., "go1.16.5"),
///   if a valid version is found and it starts with "go".
/// - `None`: If no active version is set, the file can't be read,
///   or the content doesn't represent a valid Go version (i.e., doesn't start with "go").
pub async fn get_active_version() -> Option<String> {
    let version_path = get_version_file_path();
    let active_path = version_path.join("active");

    async_fs::read_to_string(active_path)
        .await
        .ok()
        .and_then(|active_version| {
            if active_version.starts_with("go") {
                Some(active_version)
            } else {
                None
            }
        })
}

/// Checks if a given Go version is currently active in the GVM (Go Version Manager) system.
///
/// This function compares the provided version string with the currently active version
/// retrieved from the GVM configuration.
///
/// # Parameters
///
/// * `version`: A String representing the Go version to check. It should include the "go" prefix
///              (e.g., "go1.16.5").
///
/// # Returns
///
/// * `true` if the provided version matches the currently active version.
/// * `false` if the versions don't match or if there is no active version set.
pub async fn is_version_active(version: &str) -> bool {
    let active_version = get_active_version();
    active_version
        .await
        .map(|av| av == version)
        .unwrap_or(false)
}
