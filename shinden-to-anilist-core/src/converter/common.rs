use ambassador::delegatable_trait;
use chrono::{
    Datelike,
    NaiveDate,
};
use rayon::prelude::*;

use crate::converter::{
    database,
    exporter,
    extractor::TitleMetadata,
};

/// Unique identifier for an anime entry within a list or database.
///
/// This is an opaque index (backed by `usize`) used to look up entries in
/// both [`database::AnimeDatabase`] and [`crate::converter::providers::shinden::ShindenList`].
pub type AnimeId = usize;

/// Provides the metadata needed by the matching algorithm.
///
/// Implementors expose the properties of an anime entry that the
/// [`crate::converter::matcher::Matcher`] uses to score candidates against a query.
///
/// All methods except [`title`](MatchView::title) and
/// [`normalized_title`](MatchView::normalized_title) have default
/// implementations that return `None`, making them opt-in.  When a method
/// returns `None` the matcher treats that dimension as neutral (uses the
/// supplied neutral score).
///
/// # Example
///
/// ```rust,ignore
/// use shinden_to_anilist_core::common::MatchView;
///
/// struct SimpleEntry { title: String, normalized: String }
///
/// impl MatchView for SimpleEntry {
///     fn title(&self) -> &str { &self.title }
///     fn normalized_title(&self) -> &str { &self.normalized }
/// }
/// ```
pub trait MatchView {
    /// The display title of this anime entry.
    fn title(&self) -> &str;
    /// Lowercased, ASCII-normalized title used for search and comparison.
    ///
    /// Typically produced by [`crate::utils::normalize_str`].
    fn normalized_title(&self) -> &str;
    /// Extracted season / part / episode metadata from the title.
    fn title_metadata(&self) -> Option<&TitleMetadata> { None }
    /// Release year derived from [`date`](MatchView::date) when available.
    fn year(&self) -> Option<Option<i32>> { self.date().map(|d| d.map(|d| d.year())) }
    /// Premiere or release date.
    ///
    /// Returns `Some(None)` when the field is known to be absent,
    /// and `None` when the information is unavailable entirely.
    fn date(&self) -> Option<Option<NaiveDate>> { None }
    /// The anime type (TV, Movie, OVA, …) if known.
    fn anime_type(&self) -> Option<database::AnimeType> { None }
    /// The airing status (Finished, Ongoing, …) if known.
    fn status(&self) -> Option<database::AnimeStatus> { None }
    /// Total episode count, if known.
    fn episodes(&self) -> Option<i32> { None }
}

/// Provides the user-specific watch data needed for export.
///
/// Implementors supply progress, scores, and dates that are used
/// by an [`exporter::Exporter`].
///
/// Every method has a sensible default (`0`, `None`, etc.) so that
/// implementations only need to override the fields they actually track.
pub trait ExportView {
    /// Number of episodes the user has watched. Defaults to `0`.
    fn watched_episodes(&self) -> i32 { 0 }
    /// Date the user started watching. Defaults to `None`.
    fn start_date(&self) -> Option<NaiveDate> { None }
    /// Date the user finished watching. Defaults to `None`.
    fn finish_date(&self) -> Option<NaiveDate> { None }
    /// User score (typically 0–10). Defaults to `0`.
    fn score(&self) -> i32 { 0 }
    /// Current watch status (required — no default).
    fn status(&self) -> exporter::WatchStatus { exporter::WatchStatus::Completed }
    /// Free-form user notes/comments. Defaults to `None`.
    fn comments(&self) -> Option<&str> { None }
}

/// An indexed collection of anime entries, supporting both sequential and
/// parallel iteration.
///
/// This trait abstracts over any map-like structure keyed by [`AnimeId`].
/// A blanket implementation for [`indexmap::IndexMap<AnimeId, E>`] is
/// provided in the [`impls`] submodule.
///
/// # Parallel methods
///
/// Methods prefixed with `par_` or `into_par_` return [`rayon`] parallel
/// iterators.  Callers using these methods will need `rayon` as a
/// dependency.
#[delegatable_trait]
pub trait AnimeList: Send + Sync {
    /// The type of entry stored in this list.
    type Entry: Send + Sync;
    /// Iterates over all entry IDs.
    fn keys(&self) -> impl Iterator<Item = AnimeId> + '_;
    /// Parallel version of [`keys`](AnimeList::keys).
    fn par_keys(&self) -> impl ParallelIterator<Item = AnimeId> + '_;
    /// Iterates over all entries by reference.
    fn values(&self) -> impl Iterator<Item = &Self::Entry> + '_;
    /// Consumes the list and iterates over owned entries.
    fn into_values(self) -> impl Iterator<Item = Self::Entry>;
    /// Parallel version of [`values`](AnimeList::values).
    fn par_values(&self) -> impl ParallelIterator<Item = &Self::Entry> + '_;
    /// Parallel, consuming version of [`values`](AnimeList::values).
    fn into_par_values(self) -> impl IntoParallelIterator<Item = Self::Entry>;
    /// Iterates over `(id, &entry)` pairs.
    fn iter(&self) -> impl Iterator<Item = (AnimeId, &Self::Entry)> + '_;
    /// Parallel version of [`iter`](AnimeList::iter).
    fn par_iter(&self) -> impl ParallelIterator<Item = (AnimeId, &Self::Entry)> + '_;
    /// Consumes the list and iterates over `(id, entry)` pairs.
    fn into_iter(self) -> impl IntoIterator<Item = (AnimeId, Self::Entry)>;
    /// Parallel, consuming version of [`iter`](AnimeList::iter).
    fn into_par_iter(self) -> impl IntoParallelIterator<Item = (AnimeId, Self::Entry)>;
    /// Looks up an entry by its [`AnimeId`]. Returns `None` if not present.
    fn get(&self, key: AnimeId) -> Option<&Self::Entry>;
    /// Returns the number of entries in the list.
    fn len(&self) -> usize;
    /// Returns `true` when the list contains no entries.
    fn is_empty(&self) -> bool { self.len() == 0 }
}

/// Blanket [`AnimeList`] implementation for [`indexmap::IndexMap`].
pub mod impls {
    use indexmap::IndexMap;

    use super::*;

    impl<E> AnimeList for IndexMap<AnimeId, E>
    where
        E: Sync + Send,
    {
        type Entry = E;
        fn keys(&self) -> impl Iterator<Item = AnimeId> { self.keys().copied() }
        fn par_keys(&self) -> impl ParallelIterator<Item = AnimeId> { self.par_keys().copied() }
        fn values(&self) -> impl Iterator<Item = &Self::Entry> { self.values() }
        fn into_values(self) -> impl Iterator<Item = Self::Entry> { self.into_values() }
        fn par_values(&self) -> impl ParallelIterator<Item = &Self::Entry> { self.par_values() }
        fn into_par_values(self) -> impl IntoParallelIterator<Item = Self::Entry> {
            IntoParallelIterator::into_par_iter(self).map(|(_, v)| v)
        }
        fn iter(&self) -> impl Iterator<Item = (AnimeId, &Self::Entry)> { self.iter().map(|(&k, v)| (k, v)) }
        fn par_iter(&self) -> impl ParallelIterator<Item = (AnimeId, &Self::Entry)> {
            IntoParallelRefIterator::par_iter(self).map(|(&k, v)| (k, v))
        }
        fn into_iter(self) -> impl IntoIterator<Item = (AnimeId, Self::Entry)> {
            IntoIterator::into_iter(self)
        }
        fn into_par_iter(self) -> impl IntoParallelIterator<Item = (AnimeId, Self::Entry)> {
            IntoParallelIterator::into_par_iter(self)
        }
        fn get(&self, key: AnimeId) -> Option<&Self::Entry> { IndexMap::get(self, &key) }
        fn len(&self) -> usize { IndexMap::len(self) }
    }
}
