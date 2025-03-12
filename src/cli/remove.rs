use std::fs;

use crate::{error, info, success, util, Res};

/// Removes a specified version of the software from the system.
///
/// This function performs the following steps:
/// 1. Checks if the specified version is installed.
/// 2. Ensures the version is not currently active.
/// 3. Removes the default alias for the version.
/// 4. Removes the version directory.
///
/// # Parameters
///
/// * `version`: A String representing the version to be removed.
///
/// # Returns
///
/// * `Res<()>`: A Result type. Returns Ok(()) if the removal is successful,
///   or an error if any step of the removal process fails.
pub async fn remove(version: String) -> Res<()> {
    let real_version = util::get_real_version(version);

    info!("Checking if version {} is installed...", real_version);
    let installed_versions: Vec<String> = util::list_installed_versions()?;
    if !installed_versions.contains(&real_version) {
        error!(
            "Version {} is not installed. Please install it first.",
            real_version
        );
    }

    info!("Checking if version {} is active...", real_version);
    if util::is_version_active(&real_version) {
        error!(
            "Version {} is currently active. Please deactivate it first.",
            real_version
        );
    }

    info!("Removing default alias for version '{}'...", real_version);
    let alias_dir = util::get_alias_file_path();
    let alias_path = format!("{}/{}", alias_dir, "default");
    match util::remove_existing_symlink(alias_path) {
        Ok(_) => success!("Default alias removed for version {}.", real_version),
        Err(err) => error!(
            "Failed to remove default alias for version {}: {}",
            real_version, err
        ),
    }

    info!("Removing version {}...", real_version);
    let version_dir = util::get_version_file_path();
    let version_path = format!("{}/{}", version_dir, real_version);
    match fs::remove_dir_all(version_path) {
        Ok(_) => success!("Version {} removed.", real_version),
        Err(err) => error!("Failed to remove version {}: {}", real_version, err),
    }

    Ok(())
}
