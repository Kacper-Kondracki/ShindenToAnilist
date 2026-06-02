use std::{
    hint::black_box,
    time::Instant,
};

use shinden_to_anilist_core::{
    database::{
        AnimeDatabase,
        AnimeDatabaseLoad,
    },
    searcher::DefaultSearcher,
};

fn main() {
    let database = AnimeDatabase::get_from_mmap("anime-offline-database.jsonl").unwrap();

    let now = Instant::now();
    let searcher = DefaultSearcher::new(&database);
    let elapsed = now.elapsed();

    println!("took: {:.2?}", elapsed);

    black_box(searcher);
}
