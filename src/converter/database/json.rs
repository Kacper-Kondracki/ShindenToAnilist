use std::iter;

use chrono::NaiveDate;
use compact_str::CompactString;
use rayon::prelude::*;
use serde::Deserialize;

use super::models;
use crate::{
    converter::{
        common::AnimeId,
        extractor::TitleProcessor,
    },
    utils::normalize_str,
};

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(super) struct DatabaseRoot {
    last_update: NaiveDate,
    #[serde(default)]
    data: Vec<AnimeEntry>,
}

impl DatabaseRoot {
    pub(super) fn into_model(self) -> models::AnimeDatabase {
        let DatabaseRoot { last_update, data } = self;
        models::AnimeDatabase {
            last_update,
            entries: data
                .into_iter()
                .filter_map(|a| a.into_model())
                .map(|a| (a.id, a))
                .collect(),
        }
    }
    pub(super) fn par_into_model(self) -> models::AnimeDatabase {
        let DatabaseRoot { last_update, data } = self;
        models::AnimeDatabase {
            last_update,
            entries: data
                .into_par_iter()
                .filter_map(|a| a.into_model())
                .map(|a| (a.id, a))
                .collect(),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(super) struct AnimeEntry {
    sources: Vec<CompactString>,
    title: CompactString,
    #[serde(rename = "type")]
    anime_type: AnimeType,
    episodes: i32,
    status: AnimeStatus,
    anime_season: AnimeSeason,
    picture: CompactString,
    thumbnail: CompactString,
    duration: Option<Duration>,
    synonyms: Vec<CompactString>,
    studios: Vec<CompactString>,
    producers: Vec<CompactString>,
    related_anime: Vec<CompactString>,
    tags: Vec<CompactString>,
}
fn extract_id_from_mal_url(url: &str) -> Option<AnimeId> {
    url.contains("myanimelist")
        .then(|| url.rsplit("/").next().and_then(|s| s.parse::<AnimeId>().ok()))
        .flatten()
}

impl AnimeEntry {
    pub(super) fn into_model(self) -> Option<models::AnimeEntry> {
        let id = self.sources.iter().find_map(|s| extract_id_from_mal_url(s))?;
        let metadata = TitleProcessor::process(&self.title);
        let synonyms_metadata = self.synonyms.iter().map(|s| TitleProcessor::process(s)).collect();

        let metadata_list = iter::once(&metadata)
            .chain(&synonyms_metadata)
            .collect::<Vec<_>>();
        let consolidated_metadata = TitleProcessor::consolidate(&metadata_list);

        let normalized_title = normalize_str(&self.title);
        let normalized_synonyms = self.synonyms.iter().map(|s| normalize_str(s)).collect();
        Some(models::AnimeEntry {
            id,
            metadata,
            synonyms_metadata,
            consolidated_metadata,
            sources: self.sources,
            title: self.title,
            normalized_title,
            anime_type: self.anime_type.to_model(),
            episodes: self.episodes,
            status: self.status.to_model(),
            season: self.anime_season.season(),
            year: self.anime_season.year(),
            picture: self.picture,
            thumbnail: self.thumbnail,
            duration: self.duration.map(|d| d.to_seconds()),
            synonyms: self.synonyms,
            normalized_synonyms,
            studios: self.studios,
            producers: self.producers,
            related_anime: self.related_anime,
            tags: self.tags,
        })
    }
}

#[derive(Deserialize, Debug, Copy, Clone)]
pub(super) struct AnimeSeason {
    season: Season,
    year: Option<i32>,
}

impl AnimeSeason {
    fn season(&self) -> models::Season { self.season.to_model() }
    fn year(&self) -> Option<i32> { self.year }
}

#[derive(Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub(super) enum Season {
    Spring,
    Summer,
    Fall,
    Winter,
    Undefined,
}

impl Season {
    pub(super) fn to_model(self) -> models::Season {
        match self {
            Season::Spring => models::Season::Spring,
            Season::Summer => models::Season::Summer,
            Season::Fall => models::Season::Fall,
            Season::Winter => models::Season::Winter,
            Season::Undefined => models::Season::Undefined,
        }
    }
}

#[derive(Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub(super) enum AnimeType {
    Tv,
    Movie,
    Ova,
    Ona,
    Special,
    Unknown,
}

impl AnimeType {
    pub(super) fn to_model(self) -> models::AnimeType {
        match self {
            AnimeType::Tv => models::AnimeType::Tv,
            AnimeType::Movie => models::AnimeType::Movie,
            AnimeType::Ova => models::AnimeType::Ova,
            AnimeType::Ona => models::AnimeType::Ona,
            AnimeType::Special => models::AnimeType::Special,
            AnimeType::Unknown => models::AnimeType::Unknown,
        }
    }
}

#[derive(Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub(super) enum AnimeStatus {
    Finished,
    Ongoing,
    Upcoming,
    Unknown,
}

impl AnimeStatus {
    pub(super) fn to_model(self) -> models::AnimeStatus {
        match self {
            AnimeStatus::Finished => models::AnimeStatus::Finished,
            AnimeStatus::Ongoing => models::AnimeStatus::Ongoing,
            AnimeStatus::Upcoming => models::AnimeStatus::Upcoming,
            AnimeStatus::Unknown => models::AnimeStatus::Unknown,
        }
    }
}

#[derive(Deserialize, Debug, Copy, Clone)]

pub(super) struct Duration {
    value: i32,
}

impl Duration {
    pub(super) fn to_seconds(self) -> i32 { self.value }
}
