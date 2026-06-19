use std::{
    fs::File,
    io::{
        self,
        BufReader,
        BufWriter,
        Write,
    },
    path::{
        Path,
        PathBuf,
    },
};

use shinden_to_anilist_core::{
    common::AnimeList,
    database::{
        self,
        root_metadata_from_path,
        updater::{
            latest_database_archive_asset,
            update_latest_jsonl_from_github,
        },
    },
};
use tap::Conv;
use tokio::time::timeout;
use tonic::Status;
use tracing::{
    info,
    instrument,
};

use super::{
    DATABASE_DOWNLOAD_LOCK_TIMEOUT,
    ShindenToAnilist,
};
use crate::{
    DatabaseState,
    error::{
        IntoStatus,
        database_not_loaded,
        database_sidecar_io_error,
        database_sidecar_json_error,
    },
    export::{
        create_unique_temp_file,
        sync_parent_dir,
    },
    pb::*,
};

type CoreDatabaseReleaseInfo = database::updater::DatabaseReleaseInfo;

const DATABASE_SIDECAR_EXTENSION: &str = "info.json";

impl ShindenToAnilist {
    #[instrument(skip_all, name = "app.check_database_update")]
    pub async fn check_database_update(
        &self,
        request: CheckDatabaseUpdateRequest,
    ) -> Result<CheckDatabaseUpdateResponse, Status> {
        info!(path = %request.path, "checking database update");

        let remote = latest_database_archive_asset(self.http_client.clone())
            .await
            .map_err(IntoStatus::into_status)?
            .conv::<CoreDatabaseReleaseInfo>();
        let local = read_database_sidecar_blocking(&request.path).await?;
        let needs_update = local.as_ref() != Some(&remote);
        info!(path = %request.path, needs_update, "database update check finished");

        Ok(CheckDatabaseUpdateResponse {
            status: Some(database_update_check(local, remote)),
        })
    }

    #[instrument(skip_all, name = "app.download_database")]
    pub async fn download_database(
        &self,
        request: DownloadDatabaseRequest,
    ) -> Result<DownloadDatabaseResponse, Status> {
        info!(path = %request.path, "waiting for database download lock");
        let _download_guard = timeout(DATABASE_DOWNLOAD_LOCK_TIMEOUT, self.database_download_lock.lock())
            .await
            .map_err(|_| {
                Status::deadline_exceeded(format!(
                    "database download lock could not be acquired within {} seconds",
                    DATABASE_DOWNLOAD_LOCK_TIMEOUT.as_secs()
                ))
            })?;
        info!(path = %request.path, "database download lock acquired");

        let status = update_latest_jsonl_from_github(self.http_client.clone(), &request.path)
            .await
            .map_err(IntoStatus::into_status)?
            .conv::<CoreDatabaseReleaseInfo>();

        let status = write_database_sidecar_blocking(&request.path, status).await?;
        info!(path = %request.path, "database downloaded");

        Ok(DownloadDatabaseResponse {
            status: Some(status.into()),
        })
    }

    #[instrument(skip_all, name = "app.load_database")]
    pub async fn load_database(&self, request: LoadDatabaseRequest) -> Result<LoadDatabaseResponse, Status> {
        info!(path = %request.path, "loading database");

        let path = request.path;
        let database = spawn_blocking_status("database loading", move || {
            DatabaseState::load(path).map_err(IntoStatus::into_status)
        })
        .await?;
        let entries = database.database.len();

        let database_version = self.database.store(database);
        info!(database_version, entries, "database loaded");

        Ok(LoadDatabaseResponse { database_version })
    }

    #[instrument(skip_all, name = "app.get_database_metadata")]
    pub async fn get_database_metadata(
        &self,
        request: GetDatabaseMetadataRequest,
    ) -> Result<GetDatabaseMetadataResponse, Status> {
        info!(path = %request.path, "loading database metadata");

        let path = request.path;
        let metadata = spawn_blocking_status("database metadata loading", move || {
            root_metadata_from_path(path).map_err(IntoStatus::into_status)
        })
        .await?;
        info!("database metadata loaded");

        Ok(GetDatabaseMetadataResponse {
            metadata: Some(metadata.into()),
        })
    }

    #[instrument(skip_all, name = "app.get_database_ids")]
    pub async fn get_database_ids(
        &self,
        _request: GetDatabaseIdsRequest,
    ) -> Result<GetDatabaseIdsResponse, Status> {
        info!("loading database ids");

        let guard = self.database.load();

        let database_version = guard.version();
        let database = guard.get().ok_or_else(|| database_not_loaded().into_status())?;

        let ids: Vec<u64> = database.database.values().map(|entry| entry.id()).collect();
        info!(database_version, ids = ids.len(), "database ids loaded");

        Ok(GetDatabaseIdsResponse {
            database_version,
            ids,
        })
    }

    #[instrument(skip_all, name = "app.get_database_entries")]
    pub async fn get_database_entries(
        &self,
        request: GetDatabaseEntriesRequest,
    ) -> Result<GetDatabaseEntriesResponse, Status> {
        let requested_ids = request.ids.len();
        info!(requested_ids, ?request.ids, "loading database entries");

        let guard = self.database.load();

        let database_version = guard.version();
        let database = guard.get().ok_or_else(|| database_not_loaded().into_status())?;

        let entries: Vec<DatabaseEntry> = request
            .ids
            .into_iter()
            .filter_map(|id| database.database.get(id).map(DatabaseEntry::from))
            .collect();

        Ok(GetDatabaseEntriesResponse {
            database_version,
            entries,
        })
    }

    #[instrument(skip_all, name = "app.get_database_full")]
    pub async fn get_database_full(
        &self,
        _request: GetDatabaseFullRequest,
    ) -> Result<GetDatabaseFullResponse, Status> {
        let guard = self.database.load();

        let database_version = guard.version();
        let database = guard.get().ok_or_else(|| database_not_loaded().into_status())?;

        let entries = database
            .database
            .values()
            .map(DatabaseEntry::from)
            .collect::<Vec<_>>();
        info!(database_version, entries = entries.len(), "loading full database");

        Ok(GetDatabaseFullResponse {
            database_version,
            entries,
        })
    }
}

pub(super) async fn spawn_blocking_status<T>(
    operation: &'static str,
    f: impl FnOnce() -> Result<T, Status> + Send + 'static,
) -> Result<T, Status>
where
    T: Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|err| Status::internal(format!("{operation} blocking task failed: {err}")))?
}

fn database_sidecar_path(path: impl AsRef<Path>) -> PathBuf {
    let mut sidecar_path = PathBuf::from(path.as_ref());
    sidecar_path.set_extension(DATABASE_SIDECAR_EXTENSION);
    sidecar_path
}

fn read_database_sidecar(database_path: impl AsRef<Path>) -> Result<Option<CoreDatabaseReleaseInfo>, Status> {
    let database_path = database_path.as_ref();
    if !database_path.exists() {
        info!(path = %database_path.display(), "database file does not exist; no sidecar to read");
        return Ok(None);
    }

    let sidecar_path = database_sidecar_path(database_path);
    let sidecar = match File::open(&sidecar_path) {
        Ok(sidecar) => sidecar,
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            info!(path = %sidecar_path.display(), "database sidecar not found");
            return Ok(None);
        },
        Err(err) => {
            return Err(database_sidecar_io_error(err, &sidecar_path, "open").into_status());
        },
    };

    let status = serde_json::from_reader(BufReader::new(sidecar))
        .map(Some)
        .map_err(|err| database_sidecar_json_error(err, &sidecar_path).into_status())?;

    info!(path = %sidecar_path.display(), "database sidecar loaded");

    Ok(status)
}

fn write_database_sidecar(
    database_path: impl AsRef<Path>,
    status: &CoreDatabaseReleaseInfo,
) -> Result<(), Status> {
    let sidecar_path = database_sidecar_path(database_path);
    let (mut temp_file, temp_path) = create_unique_temp_file(&sidecar_path)
        .map_err(|err| database_sidecar_io_error(err, &sidecar_path, "create").into_status())?;

    let result = (|| {
        {
            let mut writer = BufWriter::new(&mut temp_file);
            serde_json::to_writer_pretty(&mut writer, status)
                .map_err(|err| database_sidecar_json_error(err, &sidecar_path).into_status())?;
            writer
                .flush()
                .map_err(|err| database_sidecar_io_error(err, &temp_path, "write").into_status())?;
        }

        temp_file
            .sync_all()
            .map_err(|err| database_sidecar_io_error(err, &temp_path, "write").into_status())?;
        drop(temp_file);

        std::fs::rename(&temp_path, &sidecar_path)
            .and_then(|_| sync_parent_dir(&sidecar_path))
            .map_err(|err| database_sidecar_io_error(err, &sidecar_path, "rename").into_status())
    })();

    if result.is_err() {
        let _ = std::fs::remove_file(&temp_path);
    }

    if result.is_ok() {
        info!(path = %sidecar_path.display(), "database sidecar written");
    }

    result
}

async fn read_database_sidecar_blocking(
    database_path: impl AsRef<Path>,
) -> Result<Option<CoreDatabaseReleaseInfo>, Status> {
    let database_path = PathBuf::from(database_path.as_ref());
    spawn_blocking_status("database sidecar read", move || {
        read_database_sidecar(database_path)
    })
    .await
}

async fn write_database_sidecar_blocking(
    database_path: impl AsRef<Path>,
    status: CoreDatabaseReleaseInfo,
) -> Result<CoreDatabaseReleaseInfo, Status> {
    let database_path = PathBuf::from(database_path.as_ref());
    spawn_blocking_status("database sidecar write", move || {
        write_database_sidecar(database_path, &status)?;
        Ok(status)
    })
    .await
}

fn database_update_check(
    local: Option<CoreDatabaseReleaseInfo>,
    remote: CoreDatabaseReleaseInfo,
) -> DatabaseUpdateCheck {
    let needs_update = local.as_ref() != Some(&remote);

    DatabaseUpdateCheck {
        local: local.map(Into::into),
        remote: Some(remote.into()),
        needs_update,
    }
}
