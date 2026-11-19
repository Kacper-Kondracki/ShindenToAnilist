mod database;
#[cfg(feature = "grpc")]
mod grpc;
mod matching;
mod providers;
mod source;
#[cfg(feature = "grpc")]
mod streaming;

use std::{
    sync::Arc,
    time::Duration,
};

use shinden_to_anilist_core::providers::shinden::ShindenList;
use tokio::sync::Mutex;

use crate::{
    DatabaseState,
    VersionedArcOption,
    http::AppHttpClients,
    source::SourceList,
};

const DATABASE_DOWNLOAD_LOCK_TIMEOUT: Duration = Duration::from_secs(10);
const SHINDEN_FETCH_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone)]
pub struct ShindenToAnilist {
    pub(super) http_clients: AppHttpClients,
    pub(super) shinden_list: VersionedArcOption<ShindenList>,
    pub(super) source_list: VersionedArcOption<SourceList>,
    pub(super) database: VersionedArcOption<DatabaseState>,
    pub(super) database_download_lock: Arc<Mutex<()>>,
}

impl ShindenToAnilist {
    pub fn new() -> Result<Self, reqwest::Error> { Ok(Self::with_http_clients(AppHttpClients::new()?)) }

    pub fn with_http_clients(http_clients: AppHttpClients) -> Self {
        Self {
            http_clients,
            shinden_list: VersionedArcOption::empty(),
            source_list: VersionedArcOption::empty(),
            database: VersionedArcOption::empty(),
            database_download_lock: Arc::new(Mutex::new(())),
        }
    }
}
