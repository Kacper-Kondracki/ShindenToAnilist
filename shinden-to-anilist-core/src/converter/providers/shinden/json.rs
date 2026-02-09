use chrono::NaiveDate;
use compact_str::CompactString;
use indexmap::IndexMap;
use rayon::prelude::*;
use serde::Deserialize;

use crate::{
    converter::{
        common::AnimeId,
        database,
        exporter,
        extractor::TitleProcessor,
        providers::shinden::models,
    },
    utils::*,
};

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(super) struct AnimeEntry {
    title_id: AnimeId,
    watch_status: WatchStatus,
    #[serde(deserialize_with = "de_bool_from_num")]
    is_favourite: bool,
    title: CompactString,
    cover_id: Option<i32>,
    #[serde(deserialize_with = "de_timestamp")]
    premiere_date: Option<NaiveDate>,
    #[serde(deserialize_with = "de_timestamp")]
    finish_date: Option<NaiveDate>,
    title_status: TitleStatus,
    episodes: Option<i32>,
    anime_type: AnimeType,
    #[serde(deserialize_with = "de_from_string")]
    watched_episodes_cnt: i32,
    rate_total: Option<i32>,
    user_note: Option<CompactString>,
    description_pl: Option<CompactString>,
}

impl AnimeEntry {
    pub(super) fn into_model(self) -> models::AnimeEntry {
        let metadata = TitleProcessor::process(&self.title);
        let normalized_title = normalize_str(&self.title);
        models::AnimeEntry {
            id: self.title_id,
            cover_id: self.cover_id,
            title: self.title,
            normalized_title,
            metadata,
            anime_status: self.title_status.to_model(),
            anime_type: self.anime_type.to_model(),
            premiere_date: self.premiere_date,
            finish_date: self.finish_date,
            episodes: self.episodes,
            is_favourite: self.is_favourite,
            watch_status: self.watch_status.to_model(),
            watched_episodes: self.watched_episodes_cnt,
            score: self.rate_total,
            note: self.user_note,
            description: self.description_pl,
        }
    }
}

#[derive(Deserialize, Copy, Clone, Debug, Eq, PartialEq)]
pub(super) enum AnimeType {
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

impl AnimeType {
    pub(super) fn to_model(self) -> database::AnimeType {
        match self {
            AnimeType::Music => database::AnimeType::Ova,
            AnimeType::Ova => database::AnimeType::Ova,
            AnimeType::Special => database::AnimeType::Special,
            AnimeType::Tv => database::AnimeType::Tv,
            AnimeType::Ona => database::AnimeType::Ona,
            AnimeType::Movie => database::AnimeType::Movie,
        }
    }
}

#[derive(Deserialize, Copy, Clone, Debug, Eq, PartialEq)]
pub(super) enum TitleStatus {
    #[serde(rename = "Finished Airing")]
    FinishedAiring,
    #[serde(rename = "Currently Airing")]
    CurrentlyAiring,
    #[serde(rename = "Proposal")]
    Proposal,
    #[serde(rename = "Not yet aired")]
    NotYetAired,
}

impl TitleStatus {
    pub(super) fn to_model(self) -> database::AnimeStatus {
        match self {
            TitleStatus::FinishedAiring => database::AnimeStatus::Finished,
            TitleStatus::CurrentlyAiring => database::AnimeStatus::Ongoing,
            TitleStatus::Proposal => database::AnimeStatus::Upcoming,
            TitleStatus::NotYetAired => database::AnimeStatus::Upcoming,
        }
    }
}

#[derive(Deserialize, Copy, Clone, Debug, Eq, PartialEq)]
pub(super) enum WatchStatus {
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

impl WatchStatus {
    pub(super) fn to_model(self) -> exporter::WatchStatus {
        match self {
            WatchStatus::Completed => exporter::WatchStatus::Completed,
            WatchStatus::Plan => exporter::WatchStatus::PlanToWatch,
            WatchStatus::InProgress => exporter::WatchStatus::Watching,
            WatchStatus::Skip => exporter::WatchStatus::Dropped,
            WatchStatus::Hold => exporter::WatchStatus::OnHold,
            WatchStatus::Dropped => exporter::WatchStatus::Dropped,
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub(super) struct AnimeList {
    // count: usize,
    items: Vec<AnimeEntry>,
}

impl AnimeList {
    fn sort(map: &mut IndexMap<AnimeId, models::AnimeEntry>) { map.sort_unstable_keys() }

    pub(super) fn into_map(self) -> IndexMap<AnimeId, models::AnimeEntry> {
        let mut map = self
            .items
            .into_iter()
            .map(|a| {
                let a = a.into_model();
                (a.id, a)
            })
            .collect();
        Self::sort(&mut map);
        map
    }
    pub(super) fn par_into_map(self) -> IndexMap<AnimeId, models::AnimeEntry> {
        let mut map = self
            .items
            .into_par_iter()
            .map(|a| {
                let a = a.into_model();
                (a.id, a)
            })
            .collect();
        Self::sort(&mut map);
        map
    }
}

#[derive(Deserialize, Debug, Clone)]
pub(super) struct Response {
    #[serde(deserialize_with = "de_bool_from_num")]
    success: bool,
    message: String,
    result: Option<AnimeList>,
}

impl Response {
    pub(super) fn try_into_model(self) -> Result<models::ShindenList, String> {
        if !self.success || self.result.is_none() {
            return Err(self.message);
        }
        let entries = self.result.unwrap().into_map();
        Ok(models::ShindenList { entries })
    }
    pub(super) fn try_par_into_model(self) -> Result<models::ShindenList, String> {
        if !self.success || self.result.is_none() {
            return Err(self.message);
        }
        let entries = self.result.unwrap().par_into_map();
        Ok(models::ShindenList { entries })
    }
}
