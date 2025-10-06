use crate::{error, success, utils, Res};

pub async fn use_version(version: String) -> Res<()> {
    let real_verison = utils::get_real_version(version);

    // get installed versions
    let installed_versions: Vec<String> = utils::list_installed_versions().await?;

    // check if version is already installed
    if !installed_versions.contains(&real_verison) {
        error!(
            "Version {} is not installed. Please install it first.",
            real_verison
        );
    }

    // check if version is already active
    if utils::is_version_active(&real_verison).await {
        success!("Version {} is already active.", real_verison);
        return Ok(());
    }

    // activate version
    utils::activate_version(real_verison).await
}
