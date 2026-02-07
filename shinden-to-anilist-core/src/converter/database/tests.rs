use std::{
    fs::File,
    io::Write,
    time::Instant,
};

use crate::{
    converter::{
        common::AnimeList,
        database::AnimeDatabase,
    },
    database::AnimeDatabaseLoad,
};

fn save_database(database: &AnimeDatabase) {
    File::options()
        .create_new(true)
        .write(true)
        .open("db-test.json")
        .and_then(|mut f| f.write_all(&serde_json::to_vec_pretty(&database).unwrap()))
        .ok();
}

#[test]
fn load_file_database_test() {
    let now = Instant::now();
    let database = AnimeDatabase::get_from_mmap("anime-offline-database.jsonl").unwrap();
    let elapsed = now.elapsed();

    println!("{} entries", database.len());
    println!("took {:.2?}", elapsed);

    save_database(&database);
}

#[test]
fn load_mmap_database_test() {
    let now = Instant::now();
    let database = AnimeDatabase::get_from_mmap("anime-offline-database.jsonl").unwrap();
    let elapsed = now.elapsed();

    println!("{} entries", database.len());
    println!("took {:.2?}", elapsed);

    save_database(&database);
}

#[test]
fn entries_database_test() {
    let database = AnimeDatabase::get_from_mmap("anime-offline-database.jsonl").unwrap();

    for entry in database.values().filter(|a| a.title.contains("Shingeki")) {
        println!("{} - {}", entry.id, entry.title);
    }

    let aot = &database[16498];
    println!("{} [{}]", aot.title, aot.synonyms.join(", "))
}
