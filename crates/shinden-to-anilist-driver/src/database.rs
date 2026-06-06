use std::path::Path;

use shinden_to_anilist_core::{
    BlockingHttpClient,
    database::{
        root_metadata_from_path,
        updater::{
            DatabaseUpdateStatus,
            update_latest_jsonl_from_github_blocking,
        },
    },
};

use crate::ffi::{
    StaDatabaseInfo,
    into_raw_string,
};

pub fn ensure_database(path: &str) -> Result<StaDatabaseInfo, String> {
    let update_status = update_latest_jsonl_from_github_blocking(BlockingHttpClient::new(), path)
        .map_err(|error| error.to_string())?;

    let metadata = root_metadata_from_path(path).map_err(|error| error.to_string())?;
    let (release, sha256, updated) = match update_status {
        DatabaseUpdateStatus::UpToDate { release, sha256 } => (release, sha256, false),
        DatabaseUpdateStatus::Updated { release, sha256, .. } => (release, sha256, true),
    };

    Ok(StaDatabaseInfo {
        last_update: into_raw_string(metadata.last_update().to_string()),
        release: into_raw_string(release),
        sha256: into_raw_string(sha256),
        path: into_raw_string(Path::new(path).display().to_string()),
        updated,
    })
}
