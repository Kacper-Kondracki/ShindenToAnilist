use crate::{
    converter::database::DatabaseRoot,
    converter::matcher::{MatchCandidate, MatcherConfig, score_shinden_candidate},
    converter::shinden,
    converter::shinden::ShindenList,
    ngram,
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
    db_map: AHashMap<u32, u32>,
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

    pub fn search(
        &self,
        query: &str,
        limit: usize,
        threshold: f32,
        use_and: bool,
    ) -> IndexMap<u32, f32> {
        let results = self.index.search::<ngram::RecallJaccard>(
            query.normalize().as_str(),
            limit,
            threshold,
            use_and,
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

    pub fn search_all(
        &self,
        query: &[&str],
        limit: usize,
        threshold: f32,
        use_and: bool,
    ) -> Vec<IndexMap<u32, f32>> {
        query
            .par_iter()
            .copied()
            .map(|x| self.search(x, limit, threshold, use_and))
            .collect()
    }

    pub fn search_shinden(
        &self,
        query: &shinden::AnimeEntry,
        limit: usize,
        threshold: f32,
        use_and: bool,
        config: Option<MatcherConfig>,
    ) -> IndexMap<u32, MatchCandidate> {
        let results = self.index.search::<ngram::RecallJaccard>(
            query.title.as_str().normalize().as_str(),
            limit,
            threshold,
            use_and,
        );

        let config = match config {
            None if use_and => MatcherConfig::and_preset(),
            None if !use_and => MatcherConfig::or_preset(),
            Some(config) => config,
            None => MatcherConfig::default(),
        };

        let (entries_results, mut cand_results) = results
            .iter()
            .copied()
            .map(|(id, score)| {
                let cand = &self.db.data[&self.db_map[&id]];
                (cand, score_shinden_candidate(query, cand, score, config))
            })
            .sorted_by(|(x, _), (y, _)| x.title.cmp(&y.title))
            .sorted_by(|(_, x), (_, y)| {
                y.score_breakdown
                    .final_score
                    .total_cmp(&x.score_breakdown.final_score)
            })
            .collect::<(Vec<_>, Vec<_>)>();

        match &mut cand_results[..] {
            [x] if x.score_breakdown.final_score >= config.single_threshold => {
                x.likely_match = true;
            }
            [x, y, rest @ ..]
                if (x.score_breakdown.final_score - y.score_breakdown.final_score)
                    >= config.delta_threshold
                    && x.score_breakdown.final_score >= config.single_threshold =>
            {
                x.likely_match = true;
                y.likely_match = false;
                rest.iter_mut().for_each(|x| x.likely_match = false);
            }
            _ => {}
        };

        entries_results
            .into_iter()
            .zip(cand_results)
            .map(|(x, y)| (x.id, y))
            .collect::<IndexMap<_, _>>()
    }

    pub fn search_shinden_all(
        &self,
        query: &ShindenList,
        limit: usize,
        threshold: f32,
        use_and: bool,
        config: Option<MatcherConfig>,
    ) -> IndexMap<u32, IndexMap<u32, MatchCandidate>> {
        query
            .items
            .par_iter()
            .map(|(&id, entry)| {
                (
                    id,
                    self.search_shinden(entry, limit, threshold, use_and, config),
                )
            })
            .collect::<IndexMap<_, IndexMap<_, _>>>()
    }
}
