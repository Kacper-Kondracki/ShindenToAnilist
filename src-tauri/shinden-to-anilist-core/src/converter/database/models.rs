use std::ops::Index;

use ambassador::Delegate;
use chrono::NaiveDate;
use indexmap::IndexMap;
use rayon::prelude::*;
use serde::{
    Deserialize,
    Serialize,
};
use url::Url;

use crate::converter::{
    common::{
        AnimeId,
        AnimeList,
        ambassador_impl_AnimeList,
    },
    database::ParallelIterator,
    extractor::TitleMetadata,
};

#[derive(Serialize, Deserialize, Debug, Clone, Delegate)]
#[delegate(AnimeList, target = "entries")]
pub struct AnimeDatabase {
    pub(super) last_update: NaiveDate,
    pub(super) entries: IndexMap<AnimeId, AnimeEntry>,
}

impl Index<AnimeId> for AnimeDatabase {
    type Output = AnimeEntry;
    fn index(&self, index: AnimeId) -> &Self::Output { self.get(index).unwrap() }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnimeEntry {
    pub(super) id: AnimeId,
    pub(super) metadata: TitleMetadata,
    pub(super) synonyms_metadata: Vec<TitleMetadata>,
    pub(super) sources: Vec<Url>,
    pub(super) title: String,
    pub(super) anime_type: Option<AnimeType>,
    pub(super) episodes: i32,
    pub(super) status: Option<AnimeStatus>,
    pub(super) season: Option<Season>,
    pub(super) year: Option<i32>,
    pub(super) picture: Url,
    pub(super) thumbnail: Url,
    pub(super) duration: Option<i32>,
    pub(super) synonyms: Vec<String>,
    pub(super) studios: Vec<String>,
    pub(super) producers: Vec<String>,
    pub(super) related_anime: Vec<Url>,
    pub(super) tags: Vec<String>,
}

impl AnimeEntry {
    pub fn id(&self) -> AnimeId { self.id }
    pub fn metadata(&self) -> &TitleMetadata { &self.metadata }
    pub fn synonyms_metadata(&self) -> &[TitleMetadata] { &self.synonyms_metadata }
    pub fn sources(&self) -> &[Url] { &self.sources }
    pub fn title(&self) -> &str { &self.title }
    pub fn anime_type(&self) -> Option<AnimeType> { self.anime_type }
    pub fn episodes(&self) -> i32 { self.episodes }
    pub fn status(&self) -> Option<AnimeStatus> { self.status }
    pub fn season(&self) -> Option<Season> { self.season }
    pub fn year(&self) -> Option<i32> { self.year }
    pub fn picture(&self) -> &Url { &self.picture }
    pub fn thumbnail(&self) -> &Url { &self.thumbnail }
    pub fn duration(&self) -> Option<i32> { self.duration }
    pub fn synonyms(&self) -> &[String] { &self.synonyms }
    pub fn studios(&self) -> &[String] { &self.studios }
    pub fn producers(&self) -> &[String] { &self.producers }
    pub fn related_anime(&self) -> &[Url] { &self.related_anime }
    pub fn tags(&self) -> &[String] { &self.tags }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq)]
pub enum Season {
    Spring,
    Summer,
    Fall,
    Winter,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq)]
pub enum AnimeType {
    Tv,
    Movie,
    Ova,
    Ona,
    Special,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq)]
pub enum AnimeStatus {
    Finished,
    Ongoing,
    Upcoming,
}
