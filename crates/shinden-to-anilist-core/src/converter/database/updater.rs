use std::{
    fs::{
        self,
        File,
    },
    io::{
        self,
        Cursor,
        Read,
    },
    path::{
        Path,
        PathBuf,
    },
};

use serde::{
    Deserialize,
    Serialize,
};
use sha2::{
    Digest,
    Sha256,
};

use crate::database::DatabaseError;

pub const ANIME_OFFLINE_DATABASE_RELEASE_API: &str =
    "https://api.github.com/repos/manami-project/anime-offline-database/releases/latest";
pub const ANIME_OFFLINE_DATABASE_ASSET: &str = "anime-offline-database.jsonl.zst";
pub const GITHUB_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

/// Metadata for the downloadable compressed database archive from the latest
/// GitHub release.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DatabaseArchiveAsset {
    /// Release tag that owns the asset.
    pub release: String,
    /// Release asset name.
    pub name: String,
    /// Browser-download URL for the `.jsonl.zst` archive.
    pub download_url: String,
    /// SHA-256 digest published by GitHub for the compressed `.jsonl.zst`
    /// archive, when present in the release API response.
    pub sha256: Option<String>,
    /// Asset size in bytes, when present in the release API response.
    pub size: Option<u64>,
}

/// Progress information emitted while downloading the compressed archive.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct DownloadProgress {
    /// Bytes downloaded so far.
    pub downloaded: u64,
    /// Expected total bytes, when the server provides it.
    pub total: Option<u64>,
}

/// Downloaded compressed database archive.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DownloadedDatabaseArchive {
    /// Release tag that owns the archive.
    pub release: String,
    /// SHA-256 of the compressed `.jsonl.zst` archive.
    pub sha256: String,
    /// Compressed `.jsonl.zst` bytes.
    pub bytes: Vec<u8>,
}

/// Result of downloading and decompressing the latest database archive.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DatabaseUpdate {
    /// Release tag that was downloaded.
    pub release: String,
    /// SHA-256 of the compressed `.jsonl.zst` archive.
    pub sha256: String,
    /// Path to the decompressed JSONL database file.
    pub path: PathBuf,
    /// Number of compressed bytes downloaded.
    pub compressed_size: u64,
}

/// Information about the release to compare against upstream.
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct DatabaseReleaseInfo {
    /// Release tag.
    pub release: String,
    /// SHA-256 of the compressed `.jsonl.zst` archive.
    pub sha256: String,
    /// Number of compressed bytes.
    pub compressed_size: u64,
}

impl From<DatabaseUpdate> for DatabaseReleaseInfo {
    fn from(value: DatabaseUpdate) -> Self {
        let DatabaseUpdate {
            release,
            sha256,
            compressed_size,
            ..
        } = value;
        Self {
            release,
            sha256,
            compressed_size,
        }
    }
}

impl From<DatabaseArchiveAsset> for DatabaseReleaseInfo {
    fn from(value: DatabaseArchiveAsset) -> Self {
        let DatabaseArchiveAsset {
            release,
            sha256,
            size,
            ..
        } = value;
        Self {
            release,
            sha256: sha256.unwrap_or_default(),
            compressed_size: size.unwrap_or_default(),
        }
    }
}

/// Downloads the latest `anime-offline-database.jsonl.zst` GitHub release asset,
/// verifies its SHA-256 digest when GitHub provides one, decompresses it, and
/// atomically writes the decompressed JSONL database into `path`.
///
/// This function is intentionally stateless: it does not create or read sidecar
/// files. Callers that want caching or update decisions can build that policy
/// externally from [`latest_database_archive_asset`] and their own state.
pub async fn update_latest_jsonl_from_github(
    client: reqwest::Client,
    path: impl AsRef<Path>,
) -> Result<DatabaseUpdate, DatabaseError> {
    update_latest_jsonl_from_github_with_progress(client, path, |_| {}).await
}

/// Like [`update_latest_jsonl_from_github`], but reports compressed download
/// progress after every received chunk.
pub async fn update_latest_jsonl_from_github_with_progress<F>(
    client: reqwest::Client,
    path: impl AsRef<Path>,
    progress: F,
) -> Result<DatabaseUpdate, DatabaseError>
where
    F: FnMut(DownloadProgress),
{
    let path = path.as_ref().to_path_buf();
    let archive = download_latest_database_archive_with_progress(client, progress).await?;
    let compressed_size = archive.bytes.len() as u64;

    decompress_zstd_to_path(&archive.bytes, &path)?;

    Ok(DatabaseUpdate {
        release: archive.release,
        sha256: archive.sha256,
        path,
        compressed_size,
    })
}

/// Blocking variant of [`update_latest_jsonl_from_github`].
pub fn update_latest_jsonl_from_github_blocking(
    client: reqwest::blocking::Client,
    path: impl AsRef<Path>,
) -> Result<DatabaseUpdate, DatabaseError> {
    update_latest_jsonl_from_github_blocking_with_progress(client, path, |_| {})
}

/// Blocking variant of [`update_latest_jsonl_from_github_with_progress`].
pub fn update_latest_jsonl_from_github_blocking_with_progress<F>(
    client: reqwest::blocking::Client,
    path: impl AsRef<Path>,
    progress: F,
) -> Result<DatabaseUpdate, DatabaseError>
where
    F: FnMut(DownloadProgress),
{
    let path = path.as_ref().to_path_buf();
    let archive = download_latest_database_archive_blocking_with_progress(client, progress)?;
    let compressed_size = archive.bytes.len() as u64;

    decompress_zstd_to_path(&archive.bytes, &path)?;

    Ok(DatabaseUpdate {
        release: archive.release,
        sha256: archive.sha256,
        path,
        compressed_size,
    })
}

/// Resolves the latest GitHub release asset metadata for
/// `anime-offline-database.jsonl.zst`.
pub async fn latest_database_archive_asset(
    client: reqwest::Client,
) -> Result<DatabaseArchiveAsset, DatabaseError> {
    let release = client
        .get(ANIME_OFFLINE_DATABASE_RELEASE_API)
        .header(reqwest::header::USER_AGENT, GITHUB_USER_AGENT)
        .send()
        .await?
        .error_for_status()?
        .json::<GitHubRelease>()
        .await?;

    archive_asset_from_release(release)
}

/// Blocking variant of [`latest_database_archive_asset`].
pub fn latest_database_archive_asset_blocking(
    client: reqwest::blocking::Client,
) -> Result<DatabaseArchiveAsset, DatabaseError> {
    let release = client
        .get(ANIME_OFFLINE_DATABASE_RELEASE_API)
        .header(reqwest::header::USER_AGENT, GITHUB_USER_AGENT)
        .send()?
        .error_for_status()?
        .json::<GitHubRelease>()?;

    archive_asset_from_release(release)
}

/// Downloads the latest compressed database archive into memory.
pub async fn download_latest_database_archive(
    client: reqwest::Client,
) -> Result<DownloadedDatabaseArchive, DatabaseError> {
    download_latest_database_archive_with_progress(client, |_| {}).await
}

/// Downloads the latest compressed database archive into memory and reports
/// compressed-byte progress after every received chunk.
pub async fn download_latest_database_archive_with_progress<F>(
    client: reqwest::Client,
    progress: F,
) -> Result<DownloadedDatabaseArchive, DatabaseError>
where
    F: FnMut(DownloadProgress),
{
    let asset = latest_database_archive_asset(client.clone()).await?;
    download_database_archive_with_progress(client, asset, progress).await
}

/// Blocking variant of [`download_latest_database_archive`].
pub fn download_latest_database_archive_blocking(
    client: reqwest::blocking::Client,
) -> Result<DownloadedDatabaseArchive, DatabaseError> {
    download_latest_database_archive_blocking_with_progress(client, |_| {})
}

/// Blocking variant of [`download_latest_database_archive_with_progress`].
pub fn download_latest_database_archive_blocking_with_progress<F>(
    client: reqwest::blocking::Client,
    progress: F,
) -> Result<DownloadedDatabaseArchive, DatabaseError>
where
    F: FnMut(DownloadProgress),
{
    let asset = latest_database_archive_asset_blocking(client.clone())?;
    download_database_archive_blocking_with_progress(client, asset, progress)
}

/// Downloads a specific compressed database archive asset into memory.
pub async fn download_database_archive(
    client: reqwest::Client,
    asset: DatabaseArchiveAsset,
) -> Result<DownloadedDatabaseArchive, DatabaseError> {
    download_database_archive_with_progress(client, asset, |_| {}).await
}

/// Downloads a specific compressed database archive asset into memory and
/// reports compressed-byte progress after every received chunk.
pub async fn download_database_archive_with_progress<F>(
    client: reqwest::Client,
    asset: DatabaseArchiveAsset,
    mut progress: F,
) -> Result<DownloadedDatabaseArchive, DatabaseError>
where
    F: FnMut(DownloadProgress),
{
    let mut response = client
        .get(&asset.download_url)
        .header(reqwest::header::USER_AGENT, GITHUB_USER_AGENT)
        .send()
        .await?
        .error_for_status()?;

    let total = response.content_length().or(asset.size);
    let mut downloaded = 0_u64;
    let mut bytes = Vec::with_capacity(total.and_then(|total| total.try_into().ok()).unwrap_or(0));

    progress(DownloadProgress { downloaded, total });

    while let Some(chunk) = response.chunk().await? {
        downloaded += chunk.len() as u64;
        bytes.extend_from_slice(&chunk);
        progress(DownloadProgress { downloaded, total });
    }

    let sha256 = verify_archive_digest(&bytes, asset.sha256)?;

    Ok(DownloadedDatabaseArchive {
        release: asset.release,
        sha256,
        bytes,
    })
}

/// Blocking variant of [`download_database_archive`].
pub fn download_database_archive_blocking(
    client: reqwest::blocking::Client,
    asset: DatabaseArchiveAsset,
) -> Result<DownloadedDatabaseArchive, DatabaseError> {
    download_database_archive_blocking_with_progress(client, asset, |_| {})
}

/// Blocking variant of [`download_database_archive_with_progress`].
pub fn download_database_archive_blocking_with_progress<F>(
    client: reqwest::blocking::Client,
    asset: DatabaseArchiveAsset,
    mut progress: F,
) -> Result<DownloadedDatabaseArchive, DatabaseError>
where
    F: FnMut(DownloadProgress),
{
    let mut response = client
        .get(&asset.download_url)
        .header(reqwest::header::USER_AGENT, GITHUB_USER_AGENT)
        .send()?
        .error_for_status()?;

    let total = response.content_length().or(asset.size);
    let mut downloaded = 0_u64;
    let mut bytes = Vec::with_capacity(total.and_then(|total| total.try_into().ok()).unwrap_or(0));
    let mut buffer = [0_u8; 64 * 1024];

    progress(DownloadProgress { downloaded, total });

    loop {
        let read = response.read(&mut buffer)?;
        if read == 0 {
            break;
        }

        downloaded += read as u64;
        bytes.extend_from_slice(&buffer[..read]);
        progress(DownloadProgress { downloaded, total });
    }

    let sha256 = verify_archive_digest(&bytes, asset.sha256)?;

    Ok(DownloadedDatabaseArchive {
        release: asset.release,
        sha256,
        bytes,
    })
}

/// Decompresses a `.zst` archive from memory and atomically writes the decoded
/// JSONL into `path`.
pub fn decompress_zstd_to_path(compressed: &[u8], path: &Path) -> Result<(), io::Error> {
    create_parent_dir(path)?;

    let (mut output, tmp_path) = create_unique_temp_file(path)?;
    let mut decoder = Cursor::new(compressed);
    let result = zstd::stream::copy_decode(&mut decoder, &mut output)
        .and_then(|_| output.sync_all())
        .and_then(|_| {
            drop(output);
            fs::rename(&tmp_path, path).and_then(|_| sync_parent_dir(path))
        });

    if result.is_err() {
        let _ = fs::remove_file(&tmp_path);
    }

    result
}

/// Computes lowercase hexadecimal SHA-256 for `bytes`.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write;
        write!(&mut hex, "{byte:02x}").expect("writing sha256 hex to string cannot fail");
    }
    hex
}

fn archive_asset_from_release(release: GitHubRelease) -> Result<DatabaseArchiveAsset, DatabaseError> {
    let asset = release
        .assets
        .into_iter()
        .find(|asset| asset.name == ANIME_OFFLINE_DATABASE_ASSET)
        .ok_or(DatabaseError::MissingReleaseAsset {
            asset: ANIME_OFFLINE_DATABASE_ASSET,
        })?;

    let sha256 = asset.sha256_digest();

    Ok(DatabaseArchiveAsset {
        release: release.tag_name,
        name: asset.name,
        download_url: asset.browser_download_url,
        sha256,
        size: asset.size,
    })
}

fn verify_archive_digest(bytes: &[u8], expected_sha256: Option<String>) -> Result<String, DatabaseError> {
    let actual = sha256_hex(bytes);

    if let Some(expected) = expected_sha256
        && !expected.eq_ignore_ascii_case(&actual)
    {
        return Err(DatabaseError::DigestMismatch { expected, actual });
    }

    Ok(actual)
}

fn create_unique_temp_file(path: &Path) -> io::Result<(File, PathBuf)> {
    let file_name = path.file_name().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "database path must include a file name",
        )
    })?;
    let parent = path.parent().filter(|path| !path.as_os_str().is_empty());

    for attempt in 0..1000 {
        let mut temp_file_name = file_name.to_os_string();
        temp_file_name.push(format!(".tmp.{}.{}", std::process::id(), attempt));

        let temp_path = parent
            .map(|parent| parent.join(&temp_file_name))
            .unwrap_or_else(|| PathBuf::from(&temp_file_name));

        match File::options().write(true).create_new(true).open(&temp_path) {
            Ok(file) => return Ok((file, temp_path)),
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(err) => return Err(err),
        }
    }

    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "could not allocate temporary database path",
    ))
}

fn create_parent_dir(path: &Path) -> Result<(), io::Error> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }

    Ok(())
}

#[cfg(unix)]
fn sync_parent_dir(path: &Path) -> io::Result<()> {
    let Some(parent) = path.parent().filter(|path| !path.as_os_str().is_empty()) else {
        return Ok(());
    };

    File::open(parent)?.sync_all()
}

#[cfg(not(unix))]
fn sync_parent_dir(_path: &Path) -> io::Result<()> { Ok(()) }

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
    size: Option<u64>,
}

impl GitHubAsset {
    fn sha256_digest(&self) -> Option<String> {
        self.digest
            .as_deref()
            .and_then(|digest| digest.strip_prefix("sha256:"))
            .map(str::to_string)
    }
}
