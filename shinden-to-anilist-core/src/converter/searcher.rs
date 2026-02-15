use std::ops::Index;

use ahash::AHashMap;
use bon::Builder;

use crate::{
    converter::common::{
        AnimeId,
        AnimeList,
        MatchView,
    },
    database,
    ngram,
    ngram::{
        NGramIndex,
        NGramIndexBuilder,
        TfIdfCosine,
    },
};

#[cfg(test)]
mod tests;

/// Trait for searching anime entries by a normalized title query.
///
/// Implementors index a set of anime titles and return matching
/// [`AnimeId`]s with their search scores.
pub trait Searcher {
    /// Searches for `query` and returns `(AnimeId, score)` pairs.
    fn search(&self, query: &str, options: Search) -> Vec<(AnimeId, f32)>;
    /// Like [`search`](Searcher::search), but resolves each [`AnimeId`]
    /// to an [`AnimeEntry`](database::AnimeEntry) reference from `database`.
    fn search_ref<'a>(
        &self,
        database: &'a impl Index<AnimeId, Output = database::AnimeEntry>,
        query: &str,
        options: Search,
    ) -> Vec<(&'a database::AnimeEntry, f32)> {
        self.search(query, options)
            .iter()
            .map(|&(id, score)| (&database[id], score))
            .collect()
    }
}

/// Configuration for a search query.
///
/// Use the builder API via [`Search::options()`] to construct:
///
/// ```rust,ignore
/// let opts = Search::options().limit(20).strict().build();
/// ```
#[derive(Builder, Debug, Clone, Copy)]
#[builder(derive(Debug, Clone))]
#[builder(start_fn = options)]
pub struct Search {
    /// Maximum number of results to return. Defaults to `50`.
    #[builder(default = 50)]
    pub limit: usize,
    /// Minimum score threshold for inclusion. Defaults to `0.65`.
    #[builder(default = 0.65)]
    pub threshold: f32,
    /// Search mode (strict or fuzzy). Defaults to [`SearchMode::Fuzzy`].
    #[builder(default = SearchMode::Fuzzy)]
    pub mode: SearchMode,
}

use crate::{
    converter::searcher::search_builder::{
        IsUnset,
        SetMode,
        State,
    },
};

impl<S: State> SearchBuilder<S> {
    /// Shorthand for `.mode(SearchMode::Fuzzy)`.
    pub fn fuzzy(self) -> SearchBuilder<SetMode<S>>
    where
        <S as State>::Mode: IsUnset,
    {
        self.mode(SearchMode::Fuzzy)
    }
    /// Shorthand for `.mode(SearchMode::Strict)`.
    pub fn strict(self) -> SearchBuilder<SetMode<S>>
    where
        <S as State>::Mode: IsUnset,
    {
        self.mode(SearchMode::Strict)
    }
}

/// Controls the precision of searching.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SearchMode {
    /// More precise for exact matching.
    Strict,
    /// More recall, fewer misses, better for fuzzy search.
    Fuzzy,
}

/// Extension trait for types implementing [`MatchView`], adding convenience
/// search methods that use the entry's own normalized title as the query.
///
/// Blanket-implemented for all `T: MatchView`.
pub trait SearcherAnimeExt: MatchView {
    /// Searches by this entry's normalized title, returning `(&self, results)`.
    fn search_by_title(&self, searcher: &impl Searcher, options: Search) -> (&Self, Vec<(AnimeId, f32)>) {
        (self, searcher.search(self.normalized_title(), options))
    }
    /// Like [`search_by_title`](SearcherAnimeExt::search_by_title), but resolves
    /// IDs to entry references.
    fn search_by_title_ref<'a>(
        &self,
        database: &'a impl Index<AnimeId, Output = database::AnimeEntry>,
        searcher: &impl Searcher,
        options: Search,
    ) -> (&Self, Vec<(&'a database::AnimeEntry, f32)>) {
        (
            self,
            self.search_by_title(searcher, options)
                .1
                .iter()
                .map(|&(id, score)| (&database[id], score))
                .collect(),
        )
    }
}
impl<T: MatchView> SearcherAnimeExt for T {}

/// Default n-gram–based searcher.
///
/// Builds a trigram at construction time, then uses it for
/// all subsequent searches. Supports both strict and fuzzy modes via
/// [`SearchMode`].
#[derive(Debug)]
pub struct DefaultSearcher {
    index: NGramIndex<3>,
    ngram_to_id: AHashMap<u32, AnimeId>,
}

impl DefaultSearcher {
    /// Constructs a new searcher by building a trigram index over every
    /// entry (including synonyms) in `database`.
    pub fn new(database: &impl AnimeList<Entry = database::AnimeEntry>) -> Self {
        let mut index_builder = NGramIndexBuilder::default();
        let mut ngram_to_id = AHashMap::new();

        for entry in database.values() {
            let ngram_id = index_builder.add_ngram(entry.normalized_title());
            ngram_to_id.insert(ngram_id, entry.id());

            for synonym in entry.normalized_synonyms() {
                index_builder.add_alias(synonym, ngram_id);
            }
        }

        Self {
            index: index_builder.build(),
            ngram_to_id,
        }
    }
}

impl Searcher for DefaultSearcher {
    fn search(&self, query: &str, options: Search) -> Vec<(AnimeId, f32)> {
        self.index
            .search::<TfIdfCosine>(
                query,
                options.limit,
                options.threshold,
                match options.mode {
                    SearchMode::Strict => ngram::SearchMode::And,
                    SearchMode::Fuzzy => ngram::SearchMode::Or,
                },
            )
            .into_iter()
            .map(|(ng, v)| (self.ngram_to_id[&ng], v))
            .collect()
    }
}
