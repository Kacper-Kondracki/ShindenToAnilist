use std::ops::Index;

use ambassador::Delegate;
use indexmap::IndexMap;
use rayon::prelude::*;
use serde::{
    Deserialize,
    Serialize,
};

use crate::converter::{
    common::{
        AnimeId,
        AnimeList,
        NaiveDate,
        Url,
        ambassador_impl_AnimeList,
    },
    database::ParallelIterator,
    extractor::TitleMetadata,
};

#[derive(Serialize, Deserialize, Debug, Clone, Delegate)]
#[delegate(AnimeList, target = "entries")]
pub struct AnimeDatabase {
    pub(crate) last_update: NaiveDate,
    pub(crate) entries: IndexMap<AnimeId, AnimeEntry>,
}

impl Index<AnimeId> for AnimeDatabase {
    type Output = AnimeEntry;
    fn index(&self, index: AnimeId) -> &Self::Output { self.get(index).unwrap() }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnimeEntry {
    pub(crate) id: AnimeId,
    pub(crate) metadata: TitleMetadata,
    pub(crate) synonyms_metadata: Vec<TitleMetadata>,
    pub(crate) sources: Vec<Url>,
    pub(crate) title: String,
    pub(crate) anime_type: Option<AnimeType>,
    pub(crate) episodes: i32,
    pub(crate) status: Option<AnimeStatus>,
    pub(crate) season: Option<Season>,
    pub(crate) year: Option<i32>,
    pub(crate) picture: Url,
    pub(crate) thumbnail: Url,
    pub(crate) duration: Option<i32>,
    pub(crate) synonyms: Vec<String>,
    pub(crate) studios: Vec<String>,
    pub(crate) producers: Vec<String>,
    pub(crate) related_anime: Vec<Url>,
    pub(crate) tags: Vec<String>,
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
