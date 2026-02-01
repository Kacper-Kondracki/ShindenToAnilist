use crate::{
    converter::database,
    converter::exporter::StatusXml,
    converter::matcher,
    converter::matcher::ExtractedMetadata,
    converter::view::MatchView,
    converter::view::{AnimeId, ExportView},
    utils::*,
};
use chrono::{Datelike, NaiveDate};
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
    pub title_id: AnimeId,
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
    pub items: IndexMap<AnimeId, AnimeEntry>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct Response {
    #[serde(deserialize_with = "de_bool_from_num")]
    success: bool,
    message: String,
    result: Option<AnimeList>,
}

impl MatchView for AnimeEntry {
    fn title(&self) -> &str {
        self.title.as_str()
    }
    fn extracted_metadata(&self) -> Option<ExtractedMetadata> {
        Some(self.metadata)
    }
    fn year(&self) -> Option<i32> {
        self.premiere_date.map(|d| d.year())
    }
    fn date(&self) -> Option<NaiveDate> {
        self.premiere_date
    }
    fn anime_type(&self) -> Option<database::AnimeType> {
        Some(self.anime_type.into())
    }
    fn status(&self) -> Option<database::AnimeStatus> {
        Some(self.title_status.into())
    }
    fn episodes(&self) -> Option<i32> {
        self.episodes
    }
}

impl ExportView for AnimeEntry {
    fn watched_episodes(&self) -> i32 {
        self.watched_episodes_cnt
    }
    fn start_date(&self) -> Option<NaiveDate> {
        self.premiere_date
    }
    fn finish_date(&self) -> Option<NaiveDate> {
        self.finish_date
    }

    fn score(&self) -> i32 {
        self.rate_total.unwrap_or_default()
    }

    fn status(&self) -> StatusXml {
        self.watch_status.into()
    }

    fn comments(&self) -> Option<&str> {
        self.user_note.as_deref()
    }
}

impl From<AnimeType> for database::AnimeType {
    fn from(value: AnimeType) -> Self {
        match value {
            AnimeType::Music => database::AnimeType::Special,
            AnimeType::Ova => database::AnimeType::Ova,
            AnimeType::Special => database::AnimeType::Special,
            AnimeType::Tv => database::AnimeType::Tv,
            AnimeType::Ona => database::AnimeType::Ona,
            AnimeType::Movie => database::AnimeType::Movie,
        }
    }
}

impl From<TitleStatus> for database::AnimeStatus {
    fn from(value: TitleStatus) -> Self {
        match value {
            TitleStatus::FinishedAiring => database::AnimeStatus::Finished,
            TitleStatus::CurrentlyAiring => database::AnimeStatus::Ongoing,
            TitleStatus::NotYetAired => database::AnimeStatus::Upcoming,
            TitleStatus::Proposal => database::AnimeStatus::Upcoming,
        }
    }
}

impl From<WatchStatus> for StatusXml {
    fn from(value: WatchStatus) -> Self {
        match value {
            WatchStatus::Completed => StatusXml::Completed,
            WatchStatus::Plan => StatusXml::PlanToWatch,
            WatchStatus::InProgress => StatusXml::Watching,
            WatchStatus::Skip => StatusXml::Dropped,
            WatchStatus::Hold => StatusXml::OnHold,
            WatchStatus::Dropped => StatusXml::Dropped,
        }
    }
}
