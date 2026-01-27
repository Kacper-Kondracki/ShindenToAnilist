use crate::converter::database;
use crate::ngram;
use crate::ngram::{NGramIndex, NGramIndexBuilder};
use crate::utils::NormalizeStr;
use ahash::AHashMap;

#[derive(Debug)]
pub struct Searcher<'db> {
    db: &'db [database::AnimeEntry],
    db_map: AHashMap<u32, &'db database::AnimeEntry>,
    index: NGramIndex<3>,
}

#[derive(Debug)]
pub struct SearchResult<'db> {
    pub item: &'db database::AnimeEntry,
    pub score: f32,
}

impl<'db> Searcher<'db> {
    pub fn new(db: &'db [database::AnimeEntry]) -> Self {
        let mut index = NGramIndexBuilder::default();
        let mut db_map = AHashMap::new();

        for entry in db {
            let id = index.add_ngram(entry.title.as_str().normalize().as_str());
            db_map.insert(id, entry);
            for synonym in &entry.synonyms {
                index.add_alias(synonym.as_str().normalize().as_str(), id);
            }
        }
        let index = index.build();

        Self { db, index, db_map }
    }

    pub fn search(&'_ self, query: &str, limit: usize, threshold: f32) -> Vec<SearchResult<'_>> {
        let results =
            self.index
                .search::<ngram::RecallJaccard>(query.normalize().as_str(), limit, threshold);

        results
            .iter()
            .copied()
            .map(|(id, score)| SearchResult {
                item: self.db_map[&id],
                score,
            })
            .collect()
    }
}
