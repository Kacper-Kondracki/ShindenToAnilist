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

/// The top-level anime database, containing all entries and the last-update
/// timestamp.
///
/// Implements [`AnimeList`] via delegation to its `entries` field, so it
/// can be used directly with [`crate::searcher::DefaultSearcher::new`] and
/// other list-accepting APIs.
#[derive(Serialize, Deserialize, Debug, Clone, Delegate)]
#[delegate(AnimeList, target = "entries")]
pub struct AnimeDatabase {
    pub(super) last_update: NaiveDate,
    pub(super) entries: IndexMap<AnimeId, AnimeEntry>,
}

/// Header metadata from the offline database root line.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq)]
pub struct DatabaseRootMetadata {
    pub(super) last_update: NaiveDate,
}

impl AnimeDatabase {
    /// Date advertised by the database source as its last update.
    pub fn last_update(&self) -> NaiveDate { self.last_update }
}

impl DatabaseRootMetadata {
    /// Date advertised by the database source as its last update.
    pub fn last_update(&self) -> NaiveDate { self.last_update }
}

/// A single anime entry from the offline database.
///
/// All fields are read-only via accessor methods.  An entry carries both
/// the original and normalized title, pre-extracted [`TitleMetadata`], and
/// aggregated [`ConsolidatedMetadata`] from all its synonyms.
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
    /// The database-internal identifier for this entry.
    pub fn id(&self) -> AnimeId { self.id }
    /// Aggregated season/part/episode metadata from title + all synonyms.
    pub fn consolidated_metadata(&self) -> ConsolidatedMetadata { self.consolidated_metadata }
    /// External source URLs (e.g. MyAnimeList, AniList links).
    pub fn sources(&self) -> &[CompactString] { &self.sources }
    /// The primary display title.
    pub fn title(&self) -> &str { &self.title }
    /// ASCII-normalized, lowercased title for search.
    pub fn normalized_title(&self) -> &str { &self.normalized_title }
    /// Extracted [`TitleMetadata`] from the primary title.
    pub fn metadata(&self) -> &TitleMetadata { &self.metadata }
    /// The anime format (TV, Movie, OVA, …).
    pub fn anime_type(&self) -> AnimeType { self.anime_type }
    /// Total number of episodes.
    pub fn episodes(&self) -> i32 { self.episodes }
    /// Current airing status.
    pub fn status(&self) -> AnimeStatus { self.status }
    /// The airing season (Spring, Summer, Fall, Winter).
    pub fn season(&self) -> Season { self.season }
    /// The premiere year, if known.
    pub fn year(&self) -> Option<i32> { self.year }
    /// URL to the cover image.
    pub fn picture(&self) -> &CompactString { &self.picture }
    /// URL to a smaller thumbnail image.
    pub fn thumbnail(&self) -> &CompactString { &self.thumbnail }
    /// Episode duration in minutes, if known.
    pub fn duration(&self) -> Option<i32> { self.duration }
    /// Alternative titles (synonyms).
    pub fn synonyms(&self) -> &[CompactString] { &self.synonyms }
    /// Normalized versions of the synonyms, for search indexing.
    pub fn normalized_synonyms(&self) -> &[CompactString] { &self.normalized_synonyms }
    /// Extracted [`TitleMetadata`] for each synonym.
    pub fn synonyms_metadata(&self) -> &[TitleMetadata] { &self.synonyms_metadata }
    /// Animation studios.
    pub fn studios(&self) -> &[CompactString] { &self.studios }
    /// Production companies / producers.
    pub fn producers(&self) -> &[CompactString] { &self.producers }
    /// URLs to related anime entries.
    pub fn related_anime(&self) -> &[CompactString] { &self.related_anime }
    /// Genre / theme tags.
    pub fn tags(&self) -> &[CompactString] { &self.tags }
}

/// The airing season of an anime.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq)]
pub enum Season {
    /// April – June.
    Spring,
    /// July – September.
    Summer,
    /// October – December.
    Fall,
    /// January – March.
    Winter,
    /// Season is not known.
    Undefined,
}

/// The format / type of anime production.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq)]
pub enum AnimeType {
    /// Standard television series.
    Tv,
    /// Theatrical film.
    Movie,
    /// Original Video Animation (direct-to-video).
    Ova,
    /// Original Net Animation (web-exclusive).
    Ona,
    /// Special episode or bonus content.
    Special,
    /// Type is not known.
    Unknown,
}

/// The airing status of an anime.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq)]
pub enum AnimeStatus {
    /// Airing has completed.
    Finished,
    /// Currently airing.
    Ongoing,
    /// Announced but not yet airing.
    Upcoming,
    /// Status is not known.
    Unknown,
}
