use std::{env, io};

use regex::Regex;

use crate::{error, info, success, utils, Res};

/// Returns the content of the initialization script for GVM (Go Version Manager).
///
/// This function generates a bash script as a string that performs the following tasks:
/// - Sets up the GVM_ROOT environment variable
/// - Generates and sources bash completion for GVM
/// - Sources the Go environment file if it exists
/// - Adds GOROOT/bin and GOPATH/bin to the PATH if they exist and are not already included
///
/// # Arguments
///
/// * `gvm_root` - A string slice that holds the path to the GVM root directory.
///
/// # Returns
///
/// A `String` containing the bash script for GVM initialization.
fn get_init_script_content(gvm_root: &str) -> String {
    format!(
        r#"
# >>> gvm initialize >>>
export GVM_ROOT="{}"
if [ -s "$HOME/.cargo/bin/gvm" ] && [ ! -f "$HOME/.bash_completions/gvm" ]; then
        gvm completions bash > "$HOME/.bash_completions/gvm"
fi

if [ -s "$GVM_ROOT/environment/go.env" ]; then
        set -a && source "$GVM_ROOT/environment/go.env" && set +a
fi

if [ -s "$GOROOT/bin" ]; then
        case ":$PATH:" in
                *:$GOROOT/bin:*)
                        ;;
                *)
                        export PATH="$GOROOT/bin:$PATH"
                        ;;
        esac
fi

if [ -s "$GOPATH/bin" ]; then
        case ":$PATH:" in
                *:$GOPATH/bin:*)
                        ;;
                *)
                        export PATH="$GOPATH/bin:$PATH"
                        ;;
        esac
fi
# <<< gvm initialize <<<
"#,
        gvm_root
    )
}

/// Creates the base directory structure for the GVM (Go Version Manager) application.
///
/// This function attempts to create several directories that are essential for GVM's operation:
/// - Alias directory
/// - Archive directory
/// - Cache directory
/// - Environment directory
/// - Package directory
/// - Version directory
///
/// For each directory, it will:
/// 1. Attempt to create the directory.
/// 2. If successful, log a success message.
/// 3. If the directory already exists, log an info message.
/// 4. If there's an error during creation, log an error message.
///
/// # Returns
///
/// Returns `Ok(())` if all directories are created successfully or already exist.
/// Returns an error if there's a problem creating any of the directories that
/// doesn't fall into the "already exists" category.
async fn create_base_directories() -> Res<()> {
    let alias_path = utils::get_alias_file_path();
    match async_fs::create_dir_all(&alias_path).await {
        Ok(_) => success!("Alias directory created successfully."),
        Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists => {
            info!("Alias directory already exists.")
        }
        Err(e) => error!("Error creating alias directory: {}", e),
    }

    let archive_path = utils::get_archive_file_path();
    match async_fs::create_dir_all(&archive_path).await {
        Ok(_) => success!("Archive directory created successfully."),
        Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists => {
            info!("Archive directory already exists.")
        }
        Err(e) => error!("Error creating archive directory: {}", e),
    }

    let cache_dir = utils::get_cache_dir();
    match async_fs::create_dir_all(&cache_dir).await {
        Ok(_) => success!("Cache directory created successfully."),
        Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists => {
            info!("Cache directory already exists.")
        }
        Err(e) => error!("Error creating cache directory: {}", e),
    }

    let environment_path = utils::get_environment_file_path();
    match async_fs::create_dir_all(&environment_path).await {
        Ok(_) => success!("Environment directory created successfully."),
        Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists => {
            info!("Environment directory already exists.")
        }
        Err(e) => error!("Error creating environment directory: {}", e),
    }

    let package_path = utils::get_package_file_path();
    match async_fs::create_dir_all(&package_path).await {
        Ok(_) => success!("Package directory created successfully."),
        Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists => {
            info!("Package directory already exists.")
        }
        Err(e) => error!("Error creating package directory: {}", e),
    }

    let version_path = utils::get_version_file_path();
    match async_fs::create_dir_all(&version_path).await {
        Ok(_) => success!("Version directory created successfully."),
        Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists => {
            info!("Version directory already exists.")
        }
        Err(e) => error!("Error creating version directory: {}", e),
    }

    Ok(())
}

/// Initializes the Go environment with an optional specific version.
///
/// This function attempts to set up the Go environment using the provided version
/// or the default version if none is specified. It handles the initialization
/// process and reports the outcome.
///
/// # Arguments
///
/// * `version` - An optional `String` specifying the Go version to initialize.
///               If `None`, the default version will be used.
///
/// # Returns
///
/// Returns a `Res<()>`, which is a custom result type. On success, it returns
/// `Ok(())`. On failure, it returns an error detailing what went wrong during
/// the initialization process.
pub async fn init() -> Res<()> {
    // currently we only support bash
    let shell = env::var("SHELL").expect("Failed to retrieve SHELL environment variable");
    if !shell.contains("bash") {
        error!("Go environment initialization is only supported for bash shells.");
    }

    info!("Creating GVM path structure ...");
    match create_base_directories().await {
        Ok(_) => success!("GVM path structure created successfully."),
        Err(e) => {
            error!("Error creating GVM path structure: {}", e);
        }
    }

    info!("Create init script for bash shell ...");
    let gvm_base_dir = utils::get_gvm_base_file_path();
    let gvm_init_file_path = gvm_base_dir.join("init-shell");
    let init_script_content = get_init_script_content(&gvm_base_dir.to_string_lossy());
    match async_fs::write(&gvm_init_file_path, init_script_content).await {
        Ok(_) => success!("Init script created successfully."),
        Err(e) => {
            error!("Error creating init script: {}", e);
        }
    }

    info!("Initialize GVM in profile ...");
    let start_marker = "# >>> gvm initialize >>>";
    let end_marker = "# <<< gvm initialize <<<";

    let shell_config_path = utils::get_shell_config_file_path()?;
    let shell_config_content = async_fs::read_to_string(&shell_config_path).await?;

    // Build a regex that matches from the start marker to the end marker (non-greedy).
    let pattern = format!(
        "{}(?s).*?{}",
        regex::escape(start_marker),
        regex::escape(end_marker)
    );
    let re = Regex::new(&pattern)?;

    if re.is_match(&shell_config_content) {
        info!("Go environment already initialized. Reload your profile to load go environment.");
    } else {
        info!("Initializing Go environment...");
        let mut new_shell_config_content = shell_config_content;
        let content = async_fs::read_to_string(&gvm_init_file_path).await?;
        if !new_shell_config_content.ends_with('\n') {
            new_shell_config_content.push('\n');
        }
        new_shell_config_content.push_str(&content);

        match async_fs::write(&shell_config_path, new_shell_config_content).await {
            Ok(_) => success!("Go environment initialized successfully."),
            Err(e) => {
                error!("Error initializing Go environment: {}", e);
            }
        }
    }

    Ok(())
}
