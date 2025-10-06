use crate::{error, info, success, utils, Res};

/// Removes a specified alias from the system.
///
/// This function attempts to remove the given alias, but will not remove the 'default' alias.
/// If the alias doesn't exist, it will inform the user and return without error.
///
/// # Arguments
///
/// * `alias` - A String representing the name of the alias to be removed.
///
/// # Returns
///
/// * `Res<()>` - A Result type. Returns Ok(()) if the operation is successful,
///               or an error if there's a problem during the removal process.
pub async fn remove_alias(alias: String) -> Res<()> {
    if alias == "default" {
        error!("Removing 'default' as alias is not allowed. Please choose a different alias.");
    }

    let available_aliases = utils::list_aliases().await?;
    if !available_aliases.contains(&alias) {
        info!("Alias {} does not exist. Nothing to remove.", alias);
        return Ok(());
    }

    info!("Removing alias {}...", alias);
    let alias_dir = utils::get_alias_file_path();
    let alias_path = alias_dir.join(&alias);

    utils::remove_existing_symlink(alias_path).await?;
    success!("Alias {} removed.", alias);

    Ok(())
}
