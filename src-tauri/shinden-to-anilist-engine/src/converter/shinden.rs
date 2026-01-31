use crate::{converter::matcher, converter::matcher::ExtractedMetadata, utils::*};
use chrono::NaiveDate;
use eyre::OptionExt;
use indexmap::IndexMap;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use std::cmp::Ordering;

pub async fn request(user: u64, limit: u64, offset: u64) -> eyre::Result<ShindenList> {
    let response: Response = reqwest::get(format!(
        "https://lista.shinden.pl/api/userlist/{}/anime?limit={}&offset={}",
        user, limit, offset
    ))
    .await?
    .json()
    .await?;

    response
        .result
        .ok_or_eyre(response.message)
        .map(|mut list| {
            list.items.sort_by(|x, y| x.title.cmp(&y.title));
            list.items
                .sort_by(|x, y| match (x.premiere_date, y.premiere_date) {
                    (None, None) => Ordering::Equal,
                    (None, Some(_)) => Ordering::Less,
                    (Some(_), None) => Ordering::Greater,
                    (Some(x), Some(y)) => y.cmp(&x),
                });

            list.items
                .par_iter_mut()
                .for_each(|x| x.metadata = matcher::extract_metadata(x.title.as_str()));

            let mut shinden_list = ShindenList::default();
            for entry in list.items {
                shinden_list.items.insert(entry.title_id, entry);
            }
            shinden_list
        })
}

pub async fn get(user: u64) -> eyre::Result<ShindenList> {
    request(user, 999999, 0).await
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AnimeEntry {
    #[serde(default)]
    pub metadata: ExtractedMetadata,
    pub title_id: u32,
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

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct AnimeList {
    pub count: usize,
    pub items: Vec<AnimeEntry>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Default)]
pub struct ShindenList {
    pub items: IndexMap<u32, AnimeEntry>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct Response {
    #[serde(deserialize_with = "de_bool_from_num")]
    success: bool,
    message: String,
    result: Option<AnimeList>,
}
