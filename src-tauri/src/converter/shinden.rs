use crate::utils::*;
use chrono::NaiveDate;
use eyre::OptionExt;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

pub async fn request(user: u64, limit: u64, offset: u64) -> eyre::Result<AnimeList> {
    let response: Response = reqwest::get(format!(
        "https://lista.shinden.pl/api/userlist/{}/anime?limit={}&offset={}",
        user, limit, offset
    ))
    .await?
    .json()
    .await?;

    response.result.ok_or_eyre(response.message)
}

pub async fn get(user: u64) -> eyre::Result<AnimeList> {
    request(user, 999999, 0).await
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct AnimeEntry {
    pub title_id: i32,
    pub watch_status: WatchStatus,
    #[serde(deserialize_with = "de_bool_from_num")]
    pub is_favourite: bool,
    pub title: SmolStr,
    pub cover_id: Option<i32>,
    #[serde(deserialize_with = "de_timestamp")]
    pub premiere_date: Option<NaiveDate>,
    #[serde(deserialize_with = "de_timestamp")]
    pub finish_date: Option<NaiveDate>,
    pub title_status: TitleStatus,
    pub episodes: Option<i32>,
    pub anime_type: AnimeType,
    #[serde(deserialize_with = "de_from_string")]
    pub watched_episodes_cnt: i32,
    pub rate_total: Option<i32>,
    pub user_note: Option<String>,
    pub description_pl: Option<String>,
}

#[derive(Deserialize, Serialize, Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum AnimeType {
    Music,
    #[serde(rename = "OVA")]
    Ova,
    Special,
    #[serde(rename = "TV")]
    Tv,
    #[serde(rename = "ONA")]
    Ona,
    Movie,
}

#[derive(Deserialize, Serialize, Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum TitleStatus {
    #[serde(rename = "Finished Airing")]
    FinishedAiring,
    #[serde(rename = "Currently Airing")]
    CurrentlyAiring,
    #[serde(rename = "Proposal")]
    Proposal,
    #[serde(rename = "Not yet aired")]
    NotYetAired,
}
#[derive(Deserialize, Serialize, Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum WatchStatus {
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "plan")]
    Plan,
    #[serde(rename = "in progress")]
    InProgress,
    #[serde(rename = "skip")]
    Skip,
    #[serde(rename = "hold")]
    Hold,
    #[serde(rename = "dropped")]
    Dropped,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
pub struct AnimeList {
    pub count: usize,
    pub items: Vec<AnimeEntry>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
pub struct Response {
    #[serde(deserialize_with = "de_bool_from_num")]
    pub success: bool,
    pub message: String,
    pub result: Option<AnimeList>,
}
