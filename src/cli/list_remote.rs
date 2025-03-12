use crate::{util, Res};

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
    let cache_dir: String = util::get_cache_dir();
    let cache_file = format!("{}/releases.json", cache_dir);

    let releases: Vec<util::FilteredRelease> =
        util::list_cached_versions(cache_file, version, stable)?;
    let installed_releases: Vec<String> = util::list_installed_versions()?;

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
