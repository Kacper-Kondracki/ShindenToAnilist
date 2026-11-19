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
            download_latest_database_archive,
            install_database_archive_with,
            latest_database_archive_asset,
        },
    },
};
use tap::Conv;
use tokio::time::timeout;
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
        ServiceError,
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
    ) -> Result<CheckDatabaseUpdateResponse, ServiceError> {
        let remote = latest_database_archive_asset(self.http_clients.database.clone())
            .await
            .map_err(|error| ServiceError::from(error).with_context("check upstream database release"))?
            .conv::<CoreDatabaseReleaseInfo>();
        let local = read_database_sidecar_blocking(&request.path).await?;

        Ok(CheckDatabaseUpdateResponse {
            status: Some(database_update_check(local, remote)),
        })
    }

    #[instrument(skip_all, name = "app.download_database")]
    pub async fn download_database(
        &self,
        request: DownloadDatabaseRequest,
    ) -> Result<DownloadDatabaseResponse, ServiceError> {
        let download_guard = timeout(
            DATABASE_DOWNLOAD_LOCK_TIMEOUT,
            self.database_download_lock.clone().lock_owned(),
        )
        .await
        .map_err(|_| {
            ServiceError::new(
                ErrorKind::DeadlineExceeded,
                format!(
                    "database download lock could not be acquired within {} seconds",
                    DATABASE_DOWNLOAD_LOCK_TIMEOUT.as_secs()
                ),
            )
        })?;

        let archive = download_latest_database_archive(self.http_clients.database.clone())
            .await
            .map_err(|error| {
                ServiceError::from(error).with_context(format!("download database to {}", request.path))
            })?;

        let path = request.path;
        let status = run_blocking("database installation", move || {
            let _download_guard = download_guard;
            install_database_archive_with(archive, &path, |update| {
                let status = update.conv::<CoreDatabaseReleaseInfo>();
                write_database_sidecar(&path, &status)?;
                Ok(status)
            })
            .map_err(|error: ServiceError| error.with_context(format!("install database at {path}")))
        })
        .await?;
        info!(release = %status.release, sha256 = %status.sha256, "database installed");

        Ok(DownloadDatabaseResponse {
            status: Some(status.into()),
        })
    }

    #[instrument(skip_all, name = "app.load_database")]
    pub async fn load_database(
        &self,
        request: LoadDatabaseRequest,
    ) -> Result<LoadDatabaseResponse, ServiceError> {
        let path = request.path;
        let database = run_blocking("database loading", move || {
            DatabaseState::load(&path)
                .map_err(|error| ServiceError::from(error).with_context(format!("load database at {path}")))
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
    ) -> Result<GetDatabaseMetadataResponse, ServiceError> {
        let path = request.path;
        let metadata = run_blocking("database metadata loading", move || {
            root_metadata_from_path(&path).map_err(|error| {
                ServiceError::from(error).with_context(format!("read database header at {path}"))
            })
        })
        .await?;

        Ok(GetDatabaseMetadataResponse {
            metadata: Some(metadata.into()),
        })
    }

    #[instrument(skip_all, name = "app.get_database_ids")]
    pub async fn get_database_ids(
        &self,
        _request: GetDatabaseIdsRequest,
    ) -> Result<GetDatabaseIdsResponse, ServiceError> {
        let guard = self.database.load();

        let database_version = guard.version();
        let database = guard.get().ok_or_else(database_not_loaded)?;

        let ids: Vec<u64> = database.database.values().map(|entry| entry.id()).collect();

        Ok(GetDatabaseIdsResponse {
            database_version,
            ids,
        })
    }

    #[instrument(skip_all, name = "app.get_database_entries")]
    pub async fn get_database_entries(
        &self,
        request: GetDatabaseEntriesRequest,
    ) -> Result<GetDatabaseEntriesResponse, ServiceError> {
        let guard = self.database.load();

        let database_version = guard.version();
        let database = guard.get().ok_or_else(database_not_loaded)?;

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
    ) -> Result<GetDatabaseFullResponse, ServiceError> {
        let guard = self.database.load();

        let database_version = guard.version();
        let database = guard.get().ok_or_else(database_not_loaded)?;

        let entries = database
            .database
            .values()
            .map(DatabaseEntry::from)
            .collect::<Vec<_>>();

        Ok(GetDatabaseFullResponse {
            database_version,
            entries,
        })
    }
}

pub(super) async fn run_blocking<T>(
    operation: &'static str,
    f: impl FnOnce() -> Result<T, ServiceError> + Send + 'static,
) -> Result<T, ServiceError>
where
    T: Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|error| ServiceError::caused_by(ErrorKind::Unknown, operation, error))?
}

fn database_sidecar_path(path: impl AsRef<Path>) -> PathBuf {
    let mut sidecar_path = PathBuf::from(path.as_ref());
    sidecar_path.set_extension(DATABASE_SIDECAR_EXTENSION);
    sidecar_path
}

fn read_database_sidecar(
    database_path: impl AsRef<Path>,
) -> Result<Option<CoreDatabaseReleaseInfo>, ServiceError> {
    let database_path = database_path.as_ref();
    if !database_path.exists() {
        return Ok(None);
    }

    let sidecar_path = database_sidecar_path(database_path);
    let sidecar = match File::open(&sidecar_path) {
        Ok(sidecar) => sidecar,
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            return Ok(None);
        },
        Err(err) => {
            return Err(database_sidecar_io_error(err, &sidecar_path, "open"));
        },
    };

    match serde_json::from_reader(BufReader::new(sidecar)) {
        Ok(status) => Ok(Some(status)),
        Err(error) if !error.is_io() => {
            database_sidecar_json_error(error, &sidecar_path).report();
            Ok(None)
        },
        Err(error) => Err(database_sidecar_json_error(error, &sidecar_path)),
    }
}

fn write_database_sidecar(
    database_path: impl AsRef<Path>,
    status: &CoreDatabaseReleaseInfo,
) -> Result<(), ServiceError> {
    let sidecar_path = database_sidecar_path(database_path);
    let (mut temp_file, temp_path) = create_unique_temp_file(&sidecar_path)
        .map_err(|err| database_sidecar_io_error(err, &sidecar_path, "create"))?;

    let result = (|| {
        {
            let mut writer = BufWriter::new(&mut temp_file);
            serde_json::to_writer_pretty(&mut writer, status)
                .map_err(|err| database_sidecar_json_error(err, &sidecar_path))?;
            writer
                .flush()
                .map_err(|err| database_sidecar_io_error(err, &temp_path, "write"))?;
        }

        temp_file
            .sync_all()
            .map_err(|err| database_sidecar_io_error(err, &temp_path, "write"))?;
        drop(temp_file);

        std::fs::rename(&temp_path, &sidecar_path)
            .and_then(|_| sync_parent_dir(&sidecar_path))
            .map_err(|err| database_sidecar_io_error(err, &sidecar_path, "rename"))
    })();

    if result.is_err() {
        let _ = std::fs::remove_file(&temp_path);
    }

    result
}

async fn read_database_sidecar_blocking(
    database_path: impl AsRef<Path>,
) -> Result<Option<CoreDatabaseReleaseInfo>, ServiceError> {
    let database_path = PathBuf::from(database_path.as_ref());
    run_blocking("database sidecar read", move || {
        read_database_sidecar(database_path)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corrupt_sidecar_allows_database_repair() {
        let dir = std::env::temp_dir().join(format!("shinden-database-sidecar-{}", std::process::id()));
        std::fs::create_dir(&dir).unwrap();
        let path = dir.join("database.jsonl");
        std::fs::write(&path, b"database").unwrap();
        std::fs::write(database_sidecar_path(&path), b"invalid sidecar").unwrap();

        let remote = CoreDatabaseReleaseInfo {
            release: "current".into(),
            sha256: "digest".into(),
            compressed_size: 1,
        };
        let local = read_database_sidecar(&path).unwrap();
        assert!(database_update_check(local, remote.clone()).needs_update);
        write_database_sidecar(&path, &remote).unwrap();
        let local = read_database_sidecar(&path).unwrap();
        assert!(!database_update_check(local, remote).needs_update);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
