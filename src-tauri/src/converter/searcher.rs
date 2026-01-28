use crate::converter::database;
use crate::ngram;
use crate::ngram::{NGramIndex, NGramIndexBuilder};
use crate::utils::NormalizeStr;
use ahash::AHashMap;

#[derive(Debug)]
pub struct Searcher {
    db_map: AHashMap<u32, usize>,
    index: NGramIndex<3>,
}

#[derive(Debug)]
pub struct SearchResult<'db> {
    pub item: &'db database::AnimeEntry,
    pub score: f32,
}

impl Searcher {
    pub fn new(db: &[database::AnimeEntry]) -> Self {
        let mut index = NGramIndexBuilder::default();
        let mut db_map = AHashMap::new();
        for (i, entry) in db.iter().enumerate() {
            let id = index.add_ngram(entry.title.as_str().normalize().as_str());
            db_map.insert(id, i);
            for synonym in &entry.synonyms {
                index.add_alias(synonym.as_str().normalize().as_str(), id);
            }
        }
        let index = index.build();

        Self { index, db_map }
    }

    pub fn search<'db>(
        &self,
        db: &'db [database::AnimeEntry],
        query: &str,
        limit: usize,
        threshold: f32,
        use_and: bool,
    ) -> Vec<SearchResult<'db>> {
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
                item: &db[self.db_map[&id]],
                score,
            })
            .collect()
    }
}
