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

use crate::converter::{
    common::{
        AnimeId,
        AnimeList,
        ExportView,
        MatchView,
        ambassador_impl_AnimeList,
    },
    database::{
        AnimeStatus,
        AnimeType,
    },
    exporter::WatchStatus,
    extractor::TitleMetadata,
};

/// A user's anime list fetched from Shinden.
///
/// Consists of a collection of [`AnimeEntry`] objects indexed by their ID.
/// Implements [`Index`](Index) for direct access.
#[derive(Serialize, Deserialize, Debug, Clone, Delegate, PartialEq)]
#[delegate(AnimeList, target = "entries")]
pub struct ShindenList {
    pub(super) entries: IndexMap<AnimeId, AnimeEntry>,
}

impl Index<AnimeId> for ShindenList {
    type Output = AnimeEntry;
    fn index(&self, index: AnimeId) -> &Self::Output { &self.entries[&index] }
}

/// A single entry in a Shinden user's anime list.
///
/// Contains both the immutable metadata about the anime (title, dates, status)
/// and the user-specific watch data (score, episodes watched, status).
///
/// Implements [`MatchView`] for matching against the database and
/// [`ExportView`] for exporting.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AnimeEntry {
    pub(super) id: AnimeId,
    pub(super) cover_id: Option<i32>,
    pub(super) title: CompactString,
    pub(super) normalized_title: CompactString,
    pub(super) metadata: TitleMetadata,
    pub(super) anime_status: AnimeStatus,
    pub(super) anime_type: AnimeType,
    pub(super) premiere_date: Option<NaiveDate>,
    pub(super) finish_date: Option<NaiveDate>,
    pub(super) episodes: Option<i32>,
    pub(super) is_favourite: bool,
    pub(super) watch_status: WatchStatus,
    pub(super) watched_episodes: i32,
    pub(super) score: Option<i32>,
    pub(super) note: Option<CompactString>,
    pub(super) description: Option<CompactString>,
}

impl AnimeEntry {
    /// The Shinden ID for this anime.
    pub fn id(&self) -> AnimeId { self.id }
    /// ID used for constructing the cover image URL.
    pub fn cover_id(&self) -> Option<i32> { self.cover_id }
    /// The primary title.
    pub fn title(&self) -> &str { &self.title }
    /// Extracted title metadata (season, part, etc.).
    pub fn metadata(&self) -> &TitleMetadata { &self.metadata }
    /// Airing status of the anime.
    pub fn anime_status(&self) -> AnimeStatus { self.anime_status }
    /// Type of the anime (TV, Movie, etc.).
    pub fn anime_type(&self) -> AnimeType { self.anime_type }
    /// Date the anime started airing.
    pub fn premiere_date(&self) -> Option<NaiveDate> { self.premiere_date }
    /// Date the anime finished airing.
    pub fn finish_date(&self) -> Option<NaiveDate> { self.finish_date }
    /// Total number of episodes.
    pub fn episodes(&self) -> Option<i32> { self.episodes }
    /// Whether the user has marked this as a favorite.
    pub fn is_favourite(&self) -> bool { self.is_favourite }
    /// The user's watch status (Watching, Completed, etc.).
    pub fn watch_status(&self) -> WatchStatus { self.watch_status }
    /// Number of episodes the user has watched.
    pub fn watched_episodes(&self) -> i32 { self.watched_episodes }
    /// User's score (0-10).
    pub fn score(&self) -> Option<i32> { self.score }
    /// User's private notes.
    pub fn note(&self) -> Option<&CompactString> { self.note.as_ref() }
    /// Description or synopsis (if fetched).
    pub fn description(&self) -> Option<&CompactString> { self.description.as_ref() }
}

impl MatchView for AnimeEntry {
    fn title(&self) -> &str { &self.title }
    fn normalized_title(&self) -> &str { &self.normalized_title }
    fn title_metadata(&self) -> Option<&TitleMetadata> { Some(&self.metadata) }
    fn date(&self) -> Option<Option<NaiveDate>> { Some(self.premiere_date) }
    fn anime_type(&self) -> Option<AnimeType> { Some(self.anime_type) }
    fn status(&self) -> Option<AnimeStatus> { Some(self.anime_status) }
    fn episodes(&self) -> Option<i32> { Some(self.episodes.unwrap_or_default()) }
}

impl ExportView for AnimeEntry {
    fn watched_episodes(&self) -> i32 { self.watched_episodes }
    fn start_date(&self) -> Option<NaiveDate> { None }
    fn finish_date(&self) -> Option<NaiveDate> { None }
    fn score(&self) -> i32 { self.score.unwrap_or_default() }
    fn status(&self) -> WatchStatus { self.watch_status }
    fn comments(&self) -> Option<&str> { self.note.as_deref() }
}
