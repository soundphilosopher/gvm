use crate::{
    error, info, success,
    utils::{self, activate_version, get_real_version},
    Res,
};
use flate2::read::GzDecoder;
use serde_json;
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};
use tar::Archive;

/// Checks if a specific version of the software is already installed.
///
/// This function determines whether a given version of the software is
/// already present in the installation directory.
///
/// # Arguments
///
/// * `version` - A String representing the version to check for installation.
///
/// # Returns
///
/// * `bool` - Returns `true` if the version is already installed, `false` otherwise.
fn version_already_installed(version: String) -> bool {
    let install_path = utils::get_version_file_path();
    let version_path = install_path.join(&version);
    version_path.exists()
}

/// Downloads a release package from the specified URL and saves it to a temporary file.
///
/// This asynchronous function fetches a release package from the given URL, saves it to a
/// temporary file, and returns the path to the saved file.
///
/// # Arguments
///
/// * `url` - A String containing the URL of the release package to download.
///
/// # Returns
///
/// * `Result<PathBuf, Box<dyn Error + Send + Sync>>` - Returns a Result which, if successful,
///   contains a PathBuf pointing to the location of the saved temporary file. If an error occurs
///   during the download or file writing process, it returns a boxed Error.
async fn download_release(url: String) -> Result<PathBuf, Box<dyn Error + Send + Sync>> {
    let package_url = url.clone();

    info!("Download package from source: {}", url);
    let response = reqwest::get(url).await?;
    if !response.status().is_success() {
        error!(
            "Error: Failed to download package. HTTP Status: {}",
            response.status()
        );
    }

    let content = response.bytes().await?;

    // write archive to temporary file
    let package_name = package_url
        .split("/")
        .last()
        .ok_or("Invalid package URL; cannot extract package name.")?;
    let archive_path = utils::get_archive_file_path();
    let archive_file = archive_path.join(&package_name);

    info!("Create temporary archive file: {}", archive_file.display());
    match async_fs::write(&archive_file, &content).await {
        Ok(_) => info!("Temporary archive file created: {}", archive_file.display()),
        Err(err) => error!("Failed to create temporary archive file: {}", err),
    }

    Ok(archive_file)
}

/// Extracts a downloaded package and sets up the release directory.
///
/// This function takes a downloaded archive file, extracts its contents to the installation
/// directory, renames the extracted directory to match the release version, and cleans up
/// temporary files.
///
/// # Arguments
///
/// * `archive_file` - A `PathBuf` representing the path to the downloaded archive file.
/// * `release` - A `util::FilteredRelease` containing information about the release being installed.
///
/// # Returns
///
/// * `Res<()>` - A Result type. Returns `Ok(())` if the extraction and setup process is successful,
///   or an error if any step fails.
fn extract_package(archive_file: PathBuf, release: utils::FilteredRelease) -> Res<()> {
    // get install path
    let install_path = utils::get_version_file_path();

    // extract package to installation directory
    let package_file = fs::File::open(&archive_file)?;
    let decompressor = GzDecoder::new(package_file);
    let mut package_archive = Archive::new(decompressor);

    info!("Extracting package to: {}", install_path.display());
    match package_archive.unpack(&install_path) {
        Ok(_) => success!("Package extracted successfully."),
        Err(e) => error!("Error: Failed to extract package: {}", e),
    }

    // create release
    let version_path = Path::new(&install_path).join(&release.version);
    let release_dir = Path::new(&install_path).join("go");

    info!("Create release directory: {}", version_path.display());
    match fs::rename(&release_dir, &version_path) {
        Ok(_) => success!("Release {} installed successfully.", release.version),
        Err(e) => error!("Error: Failed to rename release directory: {}", e),
    }

    // clean up temporary files
    info!("Clean up temporary files ...");
    match fs::remove_file(&archive_file) {
        Ok(_) => success!("Temporary files cleaned up successfully."),
        Err(e) => error!("Error: Failed to remove temporary archive file: {}", e),
    }

    Ok(())
}

pub async fn install(version: String, use_version: bool) -> Res<()> {
    let mut cache_dir: PathBuf = utils::get_cache_dir();
    cache_dir.push("release.json");
    let data = async_fs::read_to_string(&cache_dir).await?;
    let available_versions: Vec<utils::FilteredRelease> = serde_json::from_str(&data)?;

    let version_filter = get_real_version(version);

    let releases: Vec<utils::FilteredRelease> = available_versions
        .into_iter()
        .filter(|release| release.version == version_filter)
        .collect();

    if releases.len() == 0 || releases.len() > 1 {
        error!(
            "Version not found or multiple versions found in cache for version {}.",
            version_filter
        );
    }

    let release = releases.get(0).unwrap();
    info!("Installing version {} ...", release.version);

    if version_already_installed(release.version.clone()) {
        error!("Version {} is already installed.", release.version);
    }

    let archive_file = download_release(release.url.clone()).await?;

    match extract_package(archive_file, release.clone()) {
        Ok(_) => success!("Installing version {} complete.", release.version),
        Err(err) => {
            error!("Error: Failed to extract package: {}", err);
        }
    }

    if use_version {
        return activate_version(release.version.clone()).await;
    }

    Ok(())
}
