use std::io;

use thiserror::Error;

pub use self::models::*;
use crate::{
    converter::common::{
        JsonError,
        RequestError,
        TaskError,
    },
    http_client,
};

mod json;
pub mod models;

#[cfg(test)]
mod tests;

#[derive(Error, Debug)]
#[error(transparent)]
pub enum ShindenError {
    Io(#[from] io::Error),
    Json(#[from] JsonError),
    Request(#[from] RequestError),
    TaskError(#[from] TaskError),
    #[error("shinden api returned error: {0}")]
    Shinden(String),
}

pub async fn request(user: u64, limit: u64, offset: u64) -> Result<ShindenList, ShindenError> {
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

pub async fn get(user: u64) -> Result<ShindenList, ShindenError> { request(user, 99999, 0).await }
