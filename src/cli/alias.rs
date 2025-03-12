use std::fs;

use crate::{error, info, success, util, Res};

/// Creates an alias for a specific Go version or lists existing aliases.
///
/// This function creates a symbolic link (alias) for a specified Go version,
/// or lists all existing aliases if the alias name is "list" or "ls".
/// It performs several checks to ensure the alias and target version are valid
/// before creating the alias.
///
/// # Parameters
///
/// * `alias`: A `String` representing the name of the alias to be created or "list"/"ls" to list existing aliases.
/// * `target`: An `Option<String>` representing the target Go version for which the alias is being created.
///             If `None`, the default version will be used.
///
/// # Returns
///
/// Returns `Ok(())` if the alias is successfully created or the list is displayed,
/// or an error wrapped in `Res<()>` if any step fails.
pub async fn alias(alias: String, target: Option<String>) -> Res<()> {
    if alias == "default" {
        error!("Setting 'default' as alias is not allowed. Please choose a different alias.");
    }

    if alias == "list" || alias == "ls" {
        use colored::Colorize;

        let alias_list = util::list_aliases()?;
        let alias_max_length = alias_list.iter().map(|name| name.len()).max().unwrap_or(0);
        for alias_name in alias_list {
            let alias_dir = util::get_alias_file_path();
            let alias_path = format!("{}/{}", alias_dir, alias_name);
            let release_path = fs::read_link(alias_path)?;
            println!(
                "{:<width$} ~> {}",
                if alias_name == "default" {
                    alias_name.clone().cyan().bold()
                } else {
                    alias_name.clone().normal().clear()
                },
                release_path
                    .display()
                    .to_string()
                    .truecolor(128, 128, 128)
                    .italic(),
                width = alias_max_length + 1
            );
        }

        return Ok(());
    }

    let existing_aliases = util::list_aliases()?;
    if existing_aliases.contains(&alias) {
        error!(
            "Alias {} already exists. Please choose a different alias.",
            alias
        );
    }

    let release_version = util::get_real_version(target.unwrap_or_default());
    let releases = util::list_installed_versions()?;
    if !releases.contains(&release_version) {
        error!(
            "Version {} is not installed. Please install it first.",
            release_version
        );
    }

    info!(
        "Creating alias {} for version {}...",
        alias, release_version
    );
    let release_dir = util::get_version_file_path();
    let release_path = format!("{}/{}", release_dir, release_version);
    let alias_dir = util::get_alias_file_path();
    let alias_file_path = format!("{}/{}", alias_dir, alias);

    util::create_symlink(release_path, alias_file_path)?;
    success!("Alias {} created for version {}.", alias, release_version);
    Ok(())
}
