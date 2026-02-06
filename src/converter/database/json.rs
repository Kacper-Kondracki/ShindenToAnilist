use serde::Deserialize;

use super::models;
use crate::converter::{
    common::{
        AnimeId,
        NaiveDate,
        Url,
    },
    extractor::TitleProcessor,
};

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DatabaseRoot {
    pub(crate) last_update: NaiveDate,
    #[serde(default)]
    pub(crate) data: Vec<AnimeEntry>,
}

impl DatabaseRoot {
    pub(crate) fn into_model(self) -> models::AnimeDatabase {
        let DatabaseRoot { last_update, data } = self;
        models::AnimeDatabase {
            last_update,
            entries: data.into_iter().filter_map(|a| a.into_model()).map(|a| (a.id, a)).collect(),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AnimeEntry {
    pub(crate) sources: Vec<Url>,
    pub(crate) title: String,
    #[serde(rename = "type")]
    pub(crate) anime_type: AnimeType,
    pub(crate) episodes: i32,
    pub(crate) status: AnimeStatus,
    pub(crate) anime_season: AnimeSeason,
    pub(crate) picture: Url,
    pub(crate) thumbnail: Url,
    pub(crate) duration: Option<Duration>,
    pub(crate) synonyms: Vec<String>,
    pub(crate) studios: Vec<String>,
    pub(crate) producers: Vec<String>,
    pub(crate) related_anime: Vec<Url>,
    pub(crate) tags: Vec<String>,
}
pub(crate) fn extract_id_from_mal_url(url: &Url) -> Option<AnimeId> {
    url.domain()
        .is_some_and(|d| d.contains("myanimelist"))
        .then(|| {
            url.path_segments()
                .and_then(|mut p| p.next_back().and_then(|p| p.parse::<AnimeId>().ok()))
        })
        .flatten()
}

impl AnimeEntry {
    pub(crate) fn into_model(self) -> Option<models::AnimeEntry> {
        let id = self.sources.iter().find_map(extract_id_from_mal_url)?;
        let metadata = TitleProcessor::process(&self.title);
        let synonyms_metadata = self.synonyms.iter().map(|s| TitleProcessor::process(s)).collect();
        Some(models::AnimeEntry {
            id,
            metadata,
            synonyms_metadata,
            sources: self.sources,
            title: self.title,
            anime_type: self.anime_type.to_model(),
            episodes: self.episodes,
            status: self.status.to_model(),
            season: self.anime_season.season(),
            year: self.anime_season.year(),
            picture: self.picture,
            thumbnail: self.thumbnail,
            duration: self.duration.map(|d| d.to_seconds()),
            synonyms: self.synonyms,
            studios: self.studios,
            producers: self.producers,
            related_anime: self.related_anime,
            tags: self.tags,
        })
    }
}

#[derive(Deserialize, Debug, Copy, Clone)]
pub(crate) struct AnimeSeason {
    pub(crate) season: Season,
    pub(crate) year: Option<i32>,
}

impl AnimeSeason {
    pub(crate) fn season(&self) -> Option<models::Season> { self.season.to_model() }
    pub(crate) fn year(&self) -> Option<i32> { self.year }
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
    pub(super) fn to_model(self) -> Option<models::Season> {
        match self {
            Season::Spring => Some(models::Season::Spring),
            Season::Summer => Some(models::Season::Summer),
            Season::Fall => Some(models::Season::Fall),
            Season::Winter => Some(models::Season::Winter),
            Season::Undefined => None,
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
    pub(super) fn to_model(self) -> Option<models::AnimeType> {
        match self {
            AnimeType::Tv => Some(models::AnimeType::Tv),
            AnimeType::Movie => Some(models::AnimeType::Movie),
            AnimeType::Ova => Some(models::AnimeType::Ova),
            AnimeType::Ona => Some(models::AnimeType::Ona),
            AnimeType::Special => Some(models::AnimeType::Special),
            AnimeType::Unknown => None,
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
    pub(super) fn to_model(self) -> Option<models::AnimeStatus> {
        match self {
            AnimeStatus::Finished => Some(models::AnimeStatus::Finished),
            AnimeStatus::Ongoing => Some(models::AnimeStatus::Ongoing),
            AnimeStatus::Upcoming => Some(models::AnimeStatus::Upcoming),
            AnimeStatus::Unknown => None,
        }
    }
}

#[derive(Deserialize, Debug, Copy, Clone)]

pub(super) struct Duration {
    pub(super) value: i32,
}

impl Duration {
    pub(super) fn to_seconds(self) -> i32 { self.value }
}
