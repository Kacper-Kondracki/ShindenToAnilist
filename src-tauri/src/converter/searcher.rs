use crate::converter::matcher::{
    ExtractedMetadata, MatchCandidate, MatcherConfig, extract_metadata_db, score_shinden_candidate,
};
use crate::converter::{database, shinden};
use crate::ngram;
use crate::ngram::{NGramIndex, NGramIndexBuilder};
use crate::utils::NormalizeStr;
use ahash::AHashMap;
use itertools::Itertools;

#[derive(Debug, Clone)]
pub struct Searcher {
    db_map: AHashMap<u32, usize>,
    db_metadata: Vec<Vec<ExtractedMetadata>>,
    index: NGramIndex<3>,
}

#[derive(Debug, Clone)]
pub struct SearchResult<'db> {
    pub item: &'db database::AnimeEntry,
    pub score: f32,
}

impl Searcher {
    pub fn new(db: &[database::AnimeEntry]) -> Self {
        let mut index = NGramIndexBuilder::default();
        let mut db_map = AHashMap::new();
        let mut db_metadata = vec![Vec::new(); db.len()];

        for (i, entry) in db.iter().enumerate().filter(|(_, x)| {
            let (mal, ani) = x
                .sources
                .iter()
                .fold((false, false), |(mut mal, mut ani), x| {
                    mal |= x.contains("myanimelist");
                    ani |= x.contains("anilist") | true;

                    (mal, ani)
                });
            mal && ani
        }) {
            let id = index.add_ngram(entry.title.as_str().normalize().as_str());
            db_map.insert(id, i);
            db_metadata[i] = extract_metadata_db(entry);

            for synonym in &entry.synonyms {
                index.add_alias(synonym.as_str().normalize().as_str(), id);
            }
        }
        let index = index.build();

        Self {
            index,
            db_metadata,
            db_map,
        }
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
            .sorted_by(|x, y| x.item.title.cmp(&y.item.title))
            .sorted_by(|x, y| y.score.total_cmp(&x.score))
            .collect()
    }

    pub fn search_shinden<'db>(
        &self,
        db: &'db [database::AnimeEntry],
        shinden_metadata: ExtractedMetadata,
        query: &shinden::AnimeEntry,
        limit: usize,
        threshold: f32,
        use_and: bool,
    ) -> Vec<MatchCandidate<'db>> {
        let results = self.index.search::<ngram::RecallJaccard>(
            query.title.as_str().normalize().as_str(),
            limit,
            threshold,
            use_and,
        );

        results
            .iter()
            .copied()
            .map(|(id, score)| {
                score_shinden_candidate(
                    query,
                    &db[self.db_map[&id]],
                    score,
                    shinden_metadata,
                    &self.db_metadata[self.db_map[&id]],
                    MatcherConfig::default(),
                )
            })
            .sorted_by(|x, y| x.candidate.title.cmp(&y.candidate.title))
            .sorted_by(|x, y| {
                y.score_breakdown
                    .final_score
                    .total_cmp(&x.score_breakdown.final_score)
            })
            .collect()
    }
}
