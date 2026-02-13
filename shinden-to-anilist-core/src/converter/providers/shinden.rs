use std::{
    io,
    io::Read,
};

use thiserror::Error;

pub use self::models::*;
use crate::http_client;

mod json;
pub mod models;

#[cfg(test)]
mod tests;

pub trait ShindenListLoad {
    fn shinden_request(
        user: u64,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<ShindenList, ShindenError>> + Send;
    fn get_from_shinden(user: u64) -> impl Future<Output = Result<ShindenList, ShindenError>> + Send;
    fn from_reader(reader: &mut impl Read) -> Result<ShindenList, ShindenError>;
}

#[derive(Error, Debug)]
#[error(transparent)]
pub enum ShindenError {
    Io(#[from] io::Error),
    Json(#[from] serde_json::Error),
    Request(#[from] reqwest::Error),
    TaskError(#[from] tokio::task::JoinError),
    #[error("shinden api returned error: {0}")]
    Shinden(String),
}

impl ShindenListLoad for ShindenList {
    async fn shinden_request(user: u64, limit: u64, offset: u64) -> Result<ShindenList, ShindenError> {
        let client = http_client();
        let bytes = client
            .get(format!(
                "https://lista.shinden.pl/api/userlist/{}/anime?limit={}&offset={}",
                user, limit, offset
            ))
            .send()
            .await?
            .bytes()
            .await?;

        let data = tokio::task::spawn_blocking(move || {
            serde_json::from_slice::<json::Response>(&bytes).map(|r| r.try_par_into_model())
        });

        let shinden_list = data.await??.map_err(ShindenError::Shinden)?;

        Ok(shinden_list)
    }

    async fn get_from_shinden(user: u64) -> Result<ShindenList, ShindenError> {
        Self::shinden_request(user, 99999, 0).await
    }

    fn from_reader(reader: &mut impl Read) -> Result<ShindenList, ShindenError> {
        let data: json::Response = serde_json::from_reader(reader)?;
        data.try_par_into_model().map_err(ShindenError::Shinden)
    }
}
