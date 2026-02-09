use crate::{
    converter::{
        searcher,
        searcher::{
            Search,
            Searcher,
        },
    },
    database::{
        AnimeDatabase,
        AnimeDatabaseLoad,
    },
};

#[test]
fn searcher_title_test() {
    let database = AnimeDatabase::get_from_mmap("anime-offline-database.jsonl").unwrap();
    let searcher = searcher::DefaultSearcher::new(&database);
    for (entry, score) in searcher
        .search("shingeki no kyojin", Search::options().fuzzy().build())
        .into_iter()
        .map(|(k, v)| (&database[k], v))
    {
        let text = format!("[{:.2}] {}", score, entry.title());
        println!("{}", text);
    }
}
