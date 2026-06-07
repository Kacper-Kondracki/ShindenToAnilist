use shinden_to_anilist_core::{
    database::{
        AnimeDatabase,
        AnimeDatabaseLoad,
    },
    searcher::{
        DefaultSearcher,
        Search,
        Searcher,
    },
};

fn main() {
    let database = AnimeDatabase::get_from_mmap("anime-offline-database.jsonl").unwrap();
    let searcher = DefaultSearcher::new(&database);
    for (entry, score) in searcher
        .search_ref(&database, "shingeki no kyojin", Search::options().fuzzy().build())
        .into_iter()
    {
        let text = format!("[{:.2}] {}", score, entry.title());
        println!("{}", text);
    }
}
