use crate::{error, success, util, Res};

pub async fn use_version(version: String) -> Res<()> {
    let real_verison = util::get_real_version(version);

    // get installed versions
    let installed_versions: Vec<String> = util::list_installed_versions()?;

    // check if version is already installed
    if !installed_versions.contains(&real_verison) {
        error!(
            "Version {} is not installed. Please install it first.",
            real_verison
        );
    }

    // check if version is already active
    if util::is_version_active(&real_verison) {
        success!("Version {} is already active.", real_verison);
        return Ok(());
    }

    // activate version
    return util::activate_version(real_verison);
}
