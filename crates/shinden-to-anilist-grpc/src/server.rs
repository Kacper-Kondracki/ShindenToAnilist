use std::sync::Arc;

use arc_swap::ArcSwap;
use shinden_to_anilist_core::{
    common::AnimeList,
    providers::shinden::{
        ShindenError,
        ShindenList,
        ShindenListLoad,
    },
};
use tonic::{
    Request,
    Response,
    Status,
};

use crate::{
    Versioned,
    server::pb::{
        FetchShindenListRequest,
        FetchShindenListResponse,
        GetLoadedShindenListRequest,
        GetLoadedShindenListResponse,
        shinden_to_anilist_service_server::ShindenToAnilistService,
    },
};

pub mod pb {
    tonic::include_proto!("shinden_to_anilist.v1");
}

#[derive(Debug, Default)]
pub struct ShindenToAnilist {
    http_client: reqwest::Client,
    shinden_list: ArcSwap<Versioned<Option<Arc<ShindenList>>>>,
}

impl ShindenToAnilist {
    pub fn new(http_client: reqwest::Client) -> Self {
        Self {
            http_client,
            shinden_list: ArcSwap::from_pointee(Versioned::new(None)),
        }
    }
}

#[tonic::async_trait]
impl ShindenToAnilistService for ShindenToAnilist {
    async fn fetch_shinden_list(
        &self,
        request: Request<FetchShindenListRequest>,
    ) -> Result<Response<FetchShindenListResponse>, Status> {
        let request = request.into_inner();

        let shinden = ShindenList::get_from_shinden(self.http_client.clone(), request.id)
            .await
            .map_err(|err| match err {
                ShindenError::Io(error) => Status::internal(error.to_string()),
                ShindenError::Json(error) => Status::internal(error.to_string()),
                ShindenError::Request(error) => Status::internal(error.to_string()),
                ShindenError::Shinden(error) => Status::unavailable(error),
            })?;

        let shinden = Arc::new(shinden);
        let version = self
            .shinden_list
            .rcu(|old| Versioned::new_inc(old, Some(shinden.clone())))
            .version
            .wrapping_add(1);

        Ok(Response::new(FetchShindenListResponse { version }))
    }
    async fn get_loaded_shinden_list(
        &self,
        _request: Request<GetLoadedShindenListRequest>,
    ) -> Result<Response<GetLoadedShindenListResponse>, Status> {
        // let request = request.into_inner();

        let shinden = self.shinden_list.load();

        let version = shinden.version;

        let count = shinden
            .data
            .as_ref()
            .ok_or_else(|| Status::failed_precondition("shinden list is not yet loaded"))?
            .len() as u64;

        Ok(Response::new(GetLoadedShindenListResponse { version, count }))
    }
}
