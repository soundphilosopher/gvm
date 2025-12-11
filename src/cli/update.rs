use reqwest;
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    path::{Path, PathBuf},
};

use crate::{config, info, success, utils, Res};

#[derive(Serialize, Deserialize, Debug)]
struct Release {
    version: String,
    stable: bool,
    files: Vec<File>,
}

#[derive(Serialize, Deserialize, Debug)]
struct File {
    filename: String,
    os: String,
    arch: String,
    kind: String,
}

/// Fetches the list of Go releases from the official Go website.
///
/// This asynchronous function sends a GET request to the Go downloads API,
/// retrieves the JSON response containing information about all Go releases,
/// and deserializes it into a vector of `Release` structs.
///
/// # Returns
///
/// Returns a `Result` which, on success, contains a `Vec<Release>` representing
/// all available Go releases. On failure, it returns a boxed error that
/// implements `Error + Send + Sync`.
///
/// # Errors
///
/// This function will return an error if:
/// - The HTTP request fails
/// - The response cannot be deserialized into the expected format
async fn fetch_releases() -> Result<Vec<Release>, Box<dyn Error + Send + Sync>> {
    let url = "https://go.dev/dl/?mode=json&include=all";
    let rsp = reqwest::get(url).await?;
    let releases: Vec<Release> = rsp.json().await?;
    Ok(releases)
}

/// Creates a cache file containing filtered Go releases for Linux AMD64.
///
/// This asynchronous function fetches all Go releases, filters them for Linux AMD64,
/// and writes the filtered data to a cache file in JSON format.
///
/// # Parameters
///
/// * `cache_file`: A path-like parameter specifying the location where the cache file
///    should be created or updated. It can be any type that implements `AsRef<Path>`.
///
/// # Returns
///
/// Returns a `Res<()>`, which is likely an alias for `Result<(), CustomErrorType>`.
/// On success, it returns `Ok(())`. On failure, it returns an error, which could occur
/// during fetching releases, file operations, or JSON serialization.
///
/// # Errors
///
/// This function may return an error if:
/// - Fetching releases fails
/// - Creating directories fails
/// - Writing to the cache file fails
/// - JSON serialization fails
async fn create_release_cache<P: AsRef<Path>>(cache_file: P) -> Res<()> {
    info!("Fetch releases from source ...");
    let releases = fetch_releases().await?;
    let mut filtered_releases = Vec::new();

    info!("Filter releases for Linux AMD64 ...");
    for release in releases {
        for file in release.files {
            if file.os == "linux" && file.arch == "amd64" && file.filename.ends_with("tar.gz") {
                let url = format!("https://go.dev/dl/{}", file.filename);
                filtered_releases.push(utils::FilteredRelease {
                    version: release.version.clone(),
                    url,
                });
            }
        }
    }

    // Serialize the filtered data.
    let data = serde_json::to_string_pretty(&filtered_releases)?;

    // Ensure that the parent directories exist.
    info!("Ensure cache directory exists ...");
    if let Some(parent) = cache_file.as_ref().parent() {
        async_fs::create_dir_all(parent).await?;
    }

    // Write the filtered data to the cache file.
    async_fs::write(&cache_file, &data).await?;
    success!("Cached {} releases.", filtered_releases.len());
    Ok(())
}

/// Updates the local cache of Go releases.
///
/// This asynchronous function retrieves the cache directory, constructs the path
/// for the releases cache file, and then calls `create_release_cache` to fetch
/// and store the latest Go release information.
///
/// # Returns
///
/// Returns a `Res<()>`, which is likely an alias for `Result<(), CustomErrorType>`.
/// On success, it returns `Ok(())`. On failure, it returns an error, which could occur
/// during the cache creation process.
///
/// # Errors
///
/// This function may return an error if:
/// - Retrieving the cache directory fails
/// - Creating the release cache fails
pub async fn update() -> Res<()> {
    let mut cache_dir: PathBuf = utils::get_cache_dir();
    cache_dir.push(config::RELEASE_CACHE_FILE);

    Ok(create_release_cache(cache_dir).await?)
}
