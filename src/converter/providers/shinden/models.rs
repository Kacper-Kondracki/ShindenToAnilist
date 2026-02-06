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
        ExportView,
        MatchView,
        NaiveDate,
        ambassador_impl_AnimeList,
    },
    database::{
        AnimeStatus,
        AnimeType,
    },
    exporter::WatchStatus,
    extractor::TitleMetadata,
};

#[derive(Serialize, Deserialize, Debug, Clone, Delegate)]
#[delegate(AnimeList, target = "entries")]
pub struct ShindenList {
    pub(crate) entries: IndexMap<AnimeId, AnimeEntry>,
}

impl Index<AnimeId> for ShindenList {
    type Output = AnimeEntry;
    fn index(&self, index: AnimeId) -> &Self::Output { self.get(index).unwrap() }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnimeEntry {
    pub(crate) id: AnimeId,
    pub(crate) cover_id: Option<i32>,
    pub(crate) title: String,
    pub(crate) metadata: TitleMetadata,
    pub(crate) anime_status: AnimeStatus,
    pub(crate) anime_type: AnimeType,
    pub(crate) premiere_date: Option<NaiveDate>,
    pub(crate) finish_date: Option<NaiveDate>,
    pub(crate) episodes: Option<i32>,
    pub(crate) is_favourite: bool,
    pub(crate) watch_status: WatchStatus,
    pub(crate) watched_episodes: i32,
    pub(crate) score: Option<i32>,
    pub(crate) note: Option<String>,
    pub(crate) description: Option<String>,
}

impl AnimeEntry {
    pub fn id(&self) -> AnimeId { self.id }
    pub fn cover_id(&self) -> Option<i32> { self.cover_id }
    pub fn title(&self) -> &str { &self.title }
    pub fn metadata(&self) -> &TitleMetadata { &self.metadata }
    pub fn anime_status(&self) -> AnimeStatus { self.anime_status }
    pub fn anime_type(&self) -> AnimeType { self.anime_type }
    pub fn premiere_date(&self) -> Option<NaiveDate> { self.premiere_date }
    pub fn finish_date(&self) -> Option<NaiveDate> { self.finish_date }
    pub fn episodes(&self) -> Option<i32> { self.episodes }
    pub fn is_favourite(&self) -> bool { self.is_favourite }
    pub fn watch_status(&self) -> WatchStatus { self.watch_status }
    pub fn watched_episodes(&self) -> i32 { self.watched_episodes }
    pub fn score(&self) -> Option<i32> { self.score }
    pub fn note(&self) -> &Option<String> { &self.note }
    pub fn description(&self) -> &Option<String> { &self.description }
}

impl MatchView for AnimeEntry {
    fn title(&self) -> &str { &self.title }
    fn title_metadata(&self) -> Option<&TitleMetadata> { Some(&self.metadata) }
    fn date(&self) -> Option<NaiveDate> { self.premiere_date }
    fn anime_type(&self) -> Option<AnimeType> { Some(self.anime_type) }
    fn status(&self) -> Option<AnimeStatus> { Some(self.anime_status) }
    fn episodes(&self) -> Option<i32> { self.episodes }
}

impl ExportView for AnimeEntry {
    fn watched_episodes(&self) -> i32 { self.watched_episodes }
    fn start_date(&self) -> Option<NaiveDate> { self.premiere_date }
    fn finish_date(&self) -> Option<NaiveDate> { self.finish_date }
    fn score(&self) -> i32 { self.score.unwrap_or_default() }
    fn status(&self) -> WatchStatus { self.watch_status }
    fn comments(&self) -> Option<&str> { self.note.as_deref() }
}
