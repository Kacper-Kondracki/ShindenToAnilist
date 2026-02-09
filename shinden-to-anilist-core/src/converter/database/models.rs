use std::ops::Index;

use ambassador::Delegate;
use chrono::NaiveDate;
use compact_str::CompactString;
use indexmap::IndexMap;
use rayon::prelude::*;
use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    converter::{
        common::{
            AnimeId,
            AnimeList,
            ambassador_impl_AnimeList,
        },
        database::ParallelIterator,
        extractor::TitleMetadata,
    },
    extractor::ConsolidatedMetadata,
};

#[derive(Serialize, Deserialize, Debug, Clone, Delegate)]
#[delegate(AnimeList, target = "entries")]
pub struct AnimeDatabase {
    pub(super) last_update: NaiveDate,
    pub(super) entries: IndexMap<AnimeId, AnimeEntry>,
}

impl Index<AnimeId> for AnimeDatabase {
    type Output = AnimeEntry;
    fn index<'a>(&self, index: AnimeId) -> &Self::Output { self.get(index).unwrap() }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnimeEntry {
    pub(super) id: AnimeId,
    pub(super) consolidated_metadata: ConsolidatedMetadata,
    pub(super) sources: Vec<CompactString>,
    pub(super) title: CompactString,
    pub(super) normalized_title: CompactString,
    pub(super) metadata: TitleMetadata,
    pub(super) anime_type: AnimeType,
    pub(super) episodes: i32,
    pub(super) status: AnimeStatus,
    pub(super) season: Season,
    pub(super) year: Option<i32>,
    pub(super) picture: CompactString,
    pub(super) thumbnail: CompactString,
    pub(super) duration: Option<i32>,
    pub(super) synonyms: Vec<CompactString>,
    pub(super) normalized_synonyms: Vec<CompactString>,
    pub(super) synonyms_metadata: Vec<TitleMetadata>,
    pub(super) studios: Vec<CompactString>,
    pub(super) producers: Vec<CompactString>,
    pub(super) related_anime: Vec<CompactString>,
    pub(super) tags: Vec<CompactString>,
}

impl AnimeEntry {
    pub fn id(&self) -> AnimeId { self.id }
    pub fn consolidated_metadata(&self) -> ConsolidatedMetadata { self.consolidated_metadata }
    pub fn sources(&self) -> &[CompactString] { &self.sources }
    pub fn title(&self) -> &str { &self.title }
    pub fn normalized_title(&self) -> &str { &self.normalized_title }
    pub fn metadata(&self) -> &TitleMetadata { &self.metadata }
    pub fn anime_type(&self) -> AnimeType { self.anime_type }
    pub fn episodes(&self) -> i32 { self.episodes }
    pub fn status(&self) -> AnimeStatus { self.status }
    pub fn season(&self) -> Season { self.season }
    pub fn year(&self) -> Option<i32> { self.year }
    pub fn picture(&self) -> &CompactString { &self.picture }
    pub fn thumbnail(&self) -> &CompactString { &self.thumbnail }
    pub fn duration(&self) -> Option<i32> { self.duration }
    pub fn synonyms(&self) -> &[CompactString] { &self.synonyms }
    pub fn normalized_synonyms(&self) -> &[CompactString] { &self.normalized_synonyms }
    pub fn synonyms_metadata(&self) -> &[TitleMetadata] { &self.synonyms_metadata }
    pub fn studios(&self) -> &[CompactString] { &self.studios }
    pub fn producers(&self) -> &[CompactString] { &self.producers }
    pub fn related_anime(&self) -> &[CompactString] { &self.related_anime }
    pub fn tags(&self) -> &[CompactString] { &self.tags }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq)]
pub enum Season {
    Spring,
    Summer,
    Fall,
    Winter,
    Undefined,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq)]
pub enum AnimeType {
    Tv,
    Movie,
    Ova,
    Ona,
    Special,
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq)]
pub enum AnimeStatus {
    Finished,
    Ongoing,
    Upcoming,
    Unknown,
}
