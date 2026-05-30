use shinden_to_anilist_core::{
    common::MatchView,
    database::{
        AnimeDatabase,
        AnimeDatabaseLoad,
    },
    extractor::{
        TitleMetadata,
        title_processor,
    },
    matcher::{
        DefaultMatcher,
        Matcher,
    },
    searcher::{
        DefaultSearcher,
        Search,
        Searcher,
    },
    utils::normalize_str,
};

struct MockQuery {
    title: String,
    normalized_title: String,
    metadata: TitleMetadata,
}

impl MockQuery {
    fn new(query: &str) -> Self {
        let title = query.to_string();
        let normalized_title = normalize_str(query).to_string();
        let metadata = title_processor::process(query);

        Self {
            title,
            normalized_title,
            metadata,
        }
    }
}

impl MatchView for MockQuery {
    fn title(&self) -> &str { &self.title }
    fn normalized_title(&self) -> &str { &self.normalized_title }
    fn title_metadata(&self) -> Option<&TitleMetadata> { Some(&self.metadata) }
}

fn main() {
    let database = AnimeDatabase::get_from_mmap("anime-offline-database.jsonl").unwrap();
    let searcher = DefaultSearcher::new(&database);
    let matcher = DefaultMatcher {
        search_weight: 0.8,
        season_weight: 0.2,
        ..Default::default()
    };

    let osk_queries = [
        MockQuery::new("oshi no ko 1"),
        MockQuery::new("oshi no ko 2"),
        MockQuery::new("oshi no ko 3"),
    ];

    let snk_queries = [
        MockQuery::new("shingeki no kyojin 1"),
        MockQuery::new("shingeki no kyojin 2"),
        MockQuery::new("shingeki no kyojin 3"),
    ];

    for query in osk_queries.iter().chain(&snk_queries) {
        println!("=== {} ===", query.title);
        let candidates = matcher.score_candidates(
            query,
            &searcher.search_ref(
                &database,
                &query.normalized_title,
                Search::options().fuzzy().build(),
            ),
            0.0,
        );

        for (entry, score) in candidates.items_ref(&database).take(5) {
            let text = format!("[{:.2}] {}", score.final_score, entry.title());
            println!("{}", text);
        }
    }
}
