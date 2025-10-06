use crate::{utils, Res};

/// Lists installed Go versions, optionally filtered by version and stability.
///
/// This function retrieves all installed Go versions, applies filters based on the provided
/// parameters, sorts the results, and prints them to the console.
///
/// # Parameters
///
/// * `version`: An optional String that specifies a version filter. If provided, only versions
///              matching this filter will be listed. The filter can end with '*' for prefix matching.
///
/// * `stable`: A boolean flag. When set to true, only stable versions will be listed.
///
/// # Returns
///
/// Returns `Res<()>`, which is `Ok(())` if the operation succeeds, or an error if it fails.
pub async fn list(version: Option<String>, stable: bool) -> Res<()> {
    let mut releases: Vec<String> = utils::list_installed_versions().await?;

    let version_filter = version.map(|f| {
        if f.starts_with("go") {
            f
        } else {
            format!("go{}", f)
        }
    });

    releases.retain(|r: &String| {
        if stable && !utils::is_stable_version(&r) {
            return false;
        }
        if let Some(ref filter) = version_filter {
            if filter.ends_with('*') {
                let prefix = &filter[..filter.len() - 1];
                r.starts_with(prefix)
            } else {
                r == filter
            }
        } else {
            true
        }
    });

    releases.sort_by(|a, b| utils::cmp_versions(&a, &b));

    for release in releases {
        if utils::is_version_active(&release).await {
            use colored::Colorize;
            println!("{} {}", release.green().bold(), "*".yellow());
        } else {
            println!("{}", release);
        }
    }

    Ok(())
}
