use std::{
    fs::File,
    io::Write,
    time::Instant,
};

use shinden_to_anilist_core::{
    common::AnimeList,
    database::{
        AnimeDatabase,
        AnimeDatabaseLoad,
    },
};

fn main() {
    let now = Instant::now();
    let database = AnimeDatabase::get_from_mmap("anime-offline-database.jsonl").unwrap();
    let elapsed = now.elapsed();

    println!("{} entries", database.len());
    println!("took {:.2?}", elapsed);

    File::options()
        .create_new(true)
        .write(true)
        .open("db-test.json")
        .and_then(|mut f| f.write_all(&serde_json::to_vec_pretty(&database).unwrap()))
        .ok();
}
