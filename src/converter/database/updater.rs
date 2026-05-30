use std::{
    ffi::OsString,
    fs::File,
    io::{
        self,
        Cursor,
    },
    path::{
        Path,
        PathBuf,
    },
};

use serde::Deserialize;
use sha2::{
    Digest,
    Sha256,
};

use crate::database::DatabaseError;

const ANIME_OFFLINE_DATABASE_RELEASE_API: &str =
    "https://api.github.com/repos/manami-project/anime-offline-database/releases/latest";
const ANIME_OFFLINE_DATABASE_ASSET: &str = "anime-offline-database.jsonl.zst";
const GITHUB_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

/// Result of checking and downloading the latest anime-offline-database asset.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum DatabaseUpdateStatus {
    /// The local database file already matches the latest release asset hash.
    UpToDate {
        /// Latest release tag, usually `latest`.
        release: String,
        /// SHA-256 of the compressed `.jsonl.zst` release asset.
        sha256: String,
    },
    /// A newer asset was written into the target path.
    Updated {
        /// Latest release tag, usually `latest`.
        release: String,
        /// SHA-256 of the compressed `.jsonl.zst` release asset.
        sha256: String,
        /// Path to the updated database file.
        path: PathBuf,
    },
}

/// Downloads the latest `anime-offline-database.jsonl.zst` GitHub release
/// asset when the upstream SHA-256 differs from the local sidecar hash.
///
/// The asset is decompressed into `path`. A sidecar file named `<path>.sha256`
/// stores the SHA-256 of the compressed `.zst` asset, which lets later calls
/// check the latest release metadata before downloading the asset again.
pub async fn update_latest_jsonl_from_github(
    client: reqwest::Client,
    path: impl AsRef<Path>,
) -> Result<DatabaseUpdateStatus, DatabaseError> {
    let path = path.as_ref().to_path_buf();
    let update = download_latest_compressed_database_if_needed(client, &path).await?;

    match update {
        CompressedDatabaseUpdate::UpToDate { release, sha256 } => {
            Ok(DatabaseUpdateStatus::UpToDate { release, sha256 })
        },
        CompressedDatabaseUpdate::Downloaded {
            release,
            sha256,
            compressed,
        } => {
            decompress_zstd_to_path(&compressed, &path)?;
            std::fs::write(sha256_sidecar_path(&path), format!("{sha256}\n"))?;

            Ok(DatabaseUpdateStatus::Updated {
                release,
                sha256,
                path,
            })
        },
    }
}

/// Downloads the latest compressed `anime-offline-database.jsonl.zst` GitHub
/// release asset when the upstream SHA-256 differs from the local sidecar hash.
///
/// The compressed asset is written unchanged into `path`. A sidecar file named
/// `<path>.sha256` stores the SHA-256 of the compressed asset, which lets later
/// calls check the latest release metadata before downloading the asset again.
pub async fn update_latest_jsonl_zst_from_github(
    client: reqwest::Client,
    path: impl AsRef<Path>,
) -> Result<DatabaseUpdateStatus, DatabaseError> {
    let path = path.as_ref().to_path_buf();
    let update = download_latest_compressed_database_if_needed(client, &path).await?;

    match update {
        CompressedDatabaseUpdate::UpToDate { release, sha256 } => {
            Ok(DatabaseUpdateStatus::UpToDate { release, sha256 })
        },
        CompressedDatabaseUpdate::Downloaded {
            release,
            sha256,
            compressed,
        } => {
            write_bytes_to_path(&compressed, &path)?;
            std::fs::write(sha256_sidecar_path(&path), format!("{sha256}\n"))?;

            Ok(DatabaseUpdateStatus::Updated {
                release,
                sha256,
                path,
            })
        },
    }
}

enum CompressedDatabaseUpdate {
    UpToDate {
        release: String,
        sha256: String,
    },
    Downloaded {
        release: String,
        sha256: String,
        compressed: Vec<u8>,
    },
}

async fn download_latest_compressed_database_if_needed(
    client: reqwest::Client,
    path: &Path,
) -> Result<CompressedDatabaseUpdate, DatabaseError> {
    let sha256_path = sha256_sidecar_path(&path);
    let release = client
        .get(ANIME_OFFLINE_DATABASE_RELEASE_API)
        .header(reqwest::header::USER_AGENT, GITHUB_USER_AGENT)
        .send()
        .await?
        .error_for_status()?
        .json::<GitHubRelease>()
        .await?;

    let asset = release
        .assets
        .into_iter()
        .find(|asset| asset.name == ANIME_OFFLINE_DATABASE_ASSET)
        .ok_or(DatabaseError::MissingReleaseAsset {
            asset: ANIME_OFFLINE_DATABASE_ASSET,
        })?;

    let remote_sha256 = asset.sha256_digest();
    if let Some(remote_sha256) = remote_sha256.as_deref()
        && path.exists()
        && stored_sha256_matches(&sha256_path, remote_sha256)
    {
        return Ok(CompressedDatabaseUpdate::UpToDate {
            release: release.tag_name,
            sha256: remote_sha256.to_string(),
        });
    }

    let compressed = client
        .get(asset.browser_download_url)
        .header(reqwest::header::USER_AGENT, GITHUB_USER_AGENT)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?
        .to_vec();

    let actual_sha256 = sha256_hex(&compressed);
    if let Some(expected) = remote_sha256
        && expected != actual_sha256
    {
        return Err(DatabaseError::DigestMismatch {
            expected,
            actual: actual_sha256,
        });
    }

    if path.exists() && stored_sha256_matches(&sha256_path, &actual_sha256) {
        return Ok(CompressedDatabaseUpdate::UpToDate {
            release: release.tag_name,
            sha256: actual_sha256,
        });
    }

    Ok(CompressedDatabaseUpdate::Downloaded {
        release: release.tag_name,
        sha256: actual_sha256,
        compressed,
    })
}

#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
    assets: Vec<GitHubAsset>,
}

#[derive(Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
    digest: Option<String>,
}

impl GitHubAsset {
    fn sha256_digest(&self) -> Option<String> {
        self.digest
            .as_deref()
            .and_then(|digest| digest.strip_prefix("sha256:"))
            .map(str::to_string)
    }
}

fn sha256_sidecar_path(path: &Path) -> PathBuf {
    let mut sidecar: OsString = path.as_os_str().to_owned();
    sidecar.push(".sha256");
    PathBuf::from(sidecar)
}

fn stored_sha256_matches(path: &Path, expected: &str) -> bool {
    std::fs::read_to_string(path)
        .map(|stored| stored.trim().eq_ignore_ascii_case(expected))
        .unwrap_or(false)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write;
        write!(&mut hex, "{byte:02x}").expect("writing sha256 hex to string cannot fail");
    }
    hex
}

fn decompress_zstd_to_path(compressed: &[u8], path: &Path) -> Result<(), io::Error> {
    create_parent_dir(path)?;

    let tmp_path = path.with_extension("jsonl.tmp");
    let mut decoder = Cursor::new(compressed);
    let mut output = File::create(&tmp_path)?;
    zstd::stream::copy_decode(&mut decoder, &mut output)?;
    output.sync_all()?;
    drop(output);
    std::fs::rename(tmp_path, path)
}

fn write_bytes_to_path(bytes: &[u8], path: &Path) -> Result<(), io::Error> {
    create_parent_dir(path)?;

    let tmp_path = path.with_extension("zst.tmp");
    let mut output = File::create(&tmp_path)?;
    io::Write::write_all(&mut output, bytes)?;
    output.sync_all()?;
    drop(output);
    std::fs::rename(tmp_path, path)
}

fn create_parent_dir(path: &Path) -> Result<(), io::Error> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }

    Ok(())
}
