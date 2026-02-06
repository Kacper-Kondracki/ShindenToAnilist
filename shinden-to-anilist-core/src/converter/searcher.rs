use ahash::AHashMap;
use bon::Builder;

use crate::{
    converter::{
        common::{
            AnimeId,
            AnimeList,
            MatchView,
        },
        database::AnimeDatabase,
    },
    ngram,
    ngram::{
        DefaultNormalizer,
        NGramIndex,
        NGramIndexBuilder,
        RecallJaccard,
    },
};

#[cfg(test)]
mod tests;

pub trait Searcher {
    fn search(&self, query: &str, options: Search) -> Vec<(AnimeId, f32)>;
}

#[derive(Builder, Debug, Clone, Copy)]
#[builder(derive(Debug, Clone))]
#[builder(start_fn = options)]
pub struct Search {
    #[builder(default = 50)]
    pub limit: usize,
    #[builder(default = 0.65)]
    pub threshold: f32,
    #[builder(default = SearchMode::Fuzzy)]
    pub mode: SearchMode,
}

use crate::converter::searcher::search_builder::{
    IsUnset,
    SetMode,
    State,
};

impl<S: State> SearchBuilder<S> {
    pub fn fuzzy(self) -> SearchBuilder<SetMode<S>>
    where
        <S as State>::Mode: IsUnset,
    {
        self.mode(SearchMode::Fuzzy)
    }
    pub fn strict(self) -> SearchBuilder<SetMode<S>>
    where
        <S as State>::Mode: IsUnset,
    {
        self.mode(SearchMode::Strict)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SearchMode {
    Strict,
    Fuzzy,
}

pub trait SearcherAnimeExt: MatchView {
    fn search_by_title(&self, searcher: &impl Searcher, options: Search) -> Vec<(AnimeId, f32)> {
        searcher.search(self.title(), options)
    }
}
impl<T: MatchView> SearcherAnimeExt for T {}

#[derive(Debug)]
pub struct DefaultSearcher {
    index: NGramIndex<3, DefaultNormalizer>,
    ngram_to_id: AHashMap<u32, AnimeId>,
}

impl DefaultSearcher {
    pub fn new(database: &AnimeDatabase) -> Self {
        let mut index_builder = NGramIndexBuilder::default();
        let mut ngram_to_id = AHashMap::new();

        for entry in database.values() {
            let ngram_id = index_builder.add_ngram(entry.title());
            ngram_to_id.insert(ngram_id, entry.id());

            for synonym in entry.synonyms() {
                index_builder.add_alias(synonym, ngram_id);
            }
        }

        Self { index: index_builder.build(), ngram_to_id }
    }

    fn get(&self, ngram_id: u32) -> AnimeId { self.ngram_to_id[&ngram_id] }
}

impl Searcher for DefaultSearcher {
    fn search(&self, query: &str, options: Search) -> Vec<(AnimeId, f32)> {
        self.index
            .search::<RecallJaccard>(
                query,
                options.limit,
                options.threshold,
                match options.mode {
                    SearchMode::Strict => ngram::SearchMode::And,
                    SearchMode::Fuzzy => ngram::SearchMode::Or,
                },
            )
            .into_iter()
            .map(|(ng, v)| (self.get(ng), v))
            .collect()
    }
}
