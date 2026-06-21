use std::{
    collections::HashMap,
    sync::Mutex,
};

use shinden_to_anilist_grpc::ShindenToAnilist;
use tokio_util::sync::CancellationToken;

pub(crate) const PRODUCT_NAME: &str = "ShindenToAnilist";

pub(crate) struct AppState {
    pub(crate) service: ShindenToAnilist,
    pub(crate) source_fetch_cancellations: Mutex<HashMap<u64, CancellationToken>>,
}

impl AppState {
    pub(crate) fn new() -> Result<Self, reqwest::Error> {
        Ok(Self {
            service: ShindenToAnilist::new()?,
            source_fetch_cancellations: Mutex::new(HashMap::new()),
        })
    }
}
