use crate::converter::{
    database,
    searcher,
    searcher::{
        Search,
        Searcher,
    },
};

#[test]
fn searcher_title_test() {
    let database = database::get_from_mmap("anime-offline-database.jsonl").unwrap();
    let searcher = searcher::DefaultSearcher::new(&database);

    for (entry, score) in searcher
        .search("shingeki no", Search::options().strict().build())
        .into_iter()
        .map(|(k, v)| (&database[k], v))
    {
        let text = format!("[{:.2}] {}", score, entry.title());

        println!("{}", text);
    }
}
