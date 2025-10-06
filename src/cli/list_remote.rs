use std::path::PathBuf;

use crate::{utils, Res};

/// Lists remote Go versions based on the cached releases.
///
/// This function retrieves the list of Go versions from the local cache,
/// applies filtering based on the provided parameters, and prints the
/// resulting list of versions to the console.
///
/// # Parameters
///
/// * `version`: An optional string to filter versions. If provided, only
///   versions matching this filter will be displayed. The filter can be
///   an exact version or use a wildcard (e.g., "1.21.*").
///
/// * `stable`: A boolean flag. When set to `true`, only stable versions
///   will be listed.
///
/// # Returns
///
/// Returns `Res<()>`, which is `Ok(())` if the operation succeeds, or
/// an error if there's a problem reading the cache or processing the data.
pub async fn list_remote(version: Option<String>, stable: bool) -> Res<()> {
    let mut cache_file: PathBuf = utils::get_cache_dir();
    cache_file.push("releases.json");

    let releases: Vec<utils::FilteredRelease> =
        utils::list_cached_versions(cache_file, version, stable).await?;
    let installed_releases: Vec<String> = utils::list_installed_versions().await?;

    for release in releases {
        if installed_releases.contains(&release.version) {
            use colored::Colorize;
            println!("{} {}", release.version.green().bold(), "*".yellow());
        } else {
            println!("{}", release.version);
        }
    }
    Ok(())
}
