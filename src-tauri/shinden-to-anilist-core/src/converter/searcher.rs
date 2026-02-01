use crate::{
    converter::database::DatabaseRoot,
    converter::matcher::MatchCandidates,
    converter::matcher::{MatcherConfig, score_candidate},
    converter::view::{AnimeId, AnimeList, MatchView},
    ngram,
    ngram::SearchMode,
    ngram::{NGramIndex, NGramIndexBuilder},
    utils::NormalizeStr,
};
use ahash::AHashMap;
use indexmap::IndexMap;
use itertools::Itertools;
use rayon::prelude::*;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Searcher {
    db: Arc<DatabaseRoot>,
    db_map: AHashMap<u32, AnimeId>,
    index: NGramIndex<3>,
}

impl Searcher {
    pub fn new(db: Arc<DatabaseRoot>) -> Self {
        let db_entries = &db.data;
        let mut index = NGramIndexBuilder::default();
        let mut db_map = AHashMap::new();

        for (&i, entry) in db_entries.iter() {
            let id = index.add_ngram(entry.title.as_str().normalize().as_str());
            db_map.insert(id, i);

            for synonym in &entry.synonyms {
                index.add_alias(synonym.as_str().normalize().as_str(), id);
            }
        }
        let index = index.build();

        Self { index, db, db_map }
    }

    pub fn search_title(
        &self,
        query: &impl MatchView,
        limit: usize,
        threshold: f32,
        mode: SearchMode,
    ) -> AnimeList<f32> {
        let results = self.index.search::<ngram::RecallJaccard>(
            query.title().normalize().as_str(),
            limit,
            threshold,
            mode,
        );

        results
            .iter()
            .copied()
            .map(|(id, score)| (&self.db.data[&self.db_map[&id]], score))
            .sorted_by(|(x, _), (y, _)| x.title.cmp(&y.title))
            .sorted_by(|(_, x), (_, y)| y.total_cmp(x))
            .map(|(x, score)| (x.id, score))
            .collect()
    }

    pub fn search_title_list(
        &self,
        query: &AnimeList<impl MatchView + Sync>,
        limit: usize,
        threshold: f32,
        mode: SearchMode,
    ) -> AnimeList<AnimeList<f32>> {
        query
            .par_iter()
            .map(|(&id, entry)| (id, self.search_title(entry, limit, threshold, mode)))
            .collect()
    }

    pub fn search_entry(
        &self,
        query: &impl MatchView,
        limit: usize,
        threshold: f32,
        mode: SearchMode,
        config: Option<MatcherConfig>,
    ) -> MatchCandidates {
        let results = self.index.search::<ngram::RecallJaccard>(
            query.title().normalize().as_str(),
            limit,
            threshold,
            mode,
        );

        let config = match (config, mode) {
            (None, SearchMode::And) => MatcherConfig::and_preset(),
            (None, SearchMode::Or) => MatcherConfig::or_preset(),
            (Some(config), _) => config,
        };

        let mut results = results
            .iter()
            .copied()
            .map(|(id, score)| {
                let cand = &self.db.data[&self.db_map[&id]];
                (cand, score_candidate(query, cand, score, config))
            })
            .collect::<Vec<_>>();

        results.sort_by(|(x, _), (y, _)| x.title.cmp(&y.title));
        results.sort_by(|(_, x), (_, y)| {
            y.score_breakdown
                .final_score
                .total_cmp(&x.score_breakdown.final_score)
        });

        MatchCandidates::new(
            results.into_iter().map(|(x, y)| (x.id, y)).collect(),
            config,
        )
    }

    pub fn search_entries(
        &self,
        query: &AnimeList<impl MatchView + Sync>,
        limit: usize,
        threshold: f32,
        mode: SearchMode,
        config: Option<MatcherConfig>,
    ) -> AnimeList<MatchCandidates> {
        query
            .par_iter()
            .map(|(&id, entry)| (id, self.search_entry(entry, limit, threshold, mode, config)))
            .collect::<IndexMap<_, _>>()
    }
}
