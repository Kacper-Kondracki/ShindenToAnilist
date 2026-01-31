use crate::{
    converter::database::DatabaseRoot,
    converter::matcher::{MatchCandidate, MatcherConfig, score_shinden_candidate},
    converter::{database, shinden},
    ngram,
    ngram::{NGramIndex, NGramIndexBuilder},
    utils::NormalizeStr,
};
use ahash::AHashMap;
use itertools::Itertools;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Searcher {
    db: Arc<DatabaseRoot>,
    db_map: AHashMap<u32, u32>,
    index: NGramIndex<3>,
}

#[derive(Debug, Clone)]
pub struct SearchResult<'db> {
    pub item: &'db database::AnimeEntry,
    pub score: f32,
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
    ) -> Vec<SearchResult<'_>> {
        let results = self.index.search::<ngram::RecallJaccard>(
            query.normalize().as_str(),
            limit,
            threshold,
            use_and,
        );

        results
            .iter()
            .copied()
            .map(|(id, score)| SearchResult {
                item: &self.db.data[&self.db_map[&id]],
                score,
            })
            .sorted_by(|x, y| x.item.title.cmp(&y.item.title))
            .sorted_by(|x, y| y.score.total_cmp(&x.score))
            .collect()
    }

    pub fn search_shinden(
        &self,
        query: &shinden::AnimeEntry,
        limit: usize,
        threshold: f32,
        use_and: bool,
        config: MatcherConfig,
    ) -> Vec<MatchCandidate<'_>> {
        let results = self.index.search::<ngram::RecallJaccard>(
            query.title.as_str().normalize().as_str(),
            limit,
            threshold,
            use_and,
        );

        let mut results = results
            .iter()
            .copied()
            .map(|(id, score)| {
                score_shinden_candidate(query, &self.db.data[&self.db_map[&id]], score, config)
            })
            .sorted_by(|x, y| x.candidate.title.cmp(&y.candidate.title))
            .sorted_by(|x, y| {
                y.score_breakdown
                    .final_score
                    .total_cmp(&x.score_breakdown.final_score)
            })
            .collect::<Vec<_>>();

        match &mut results[..] {
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

        results
    }
}
