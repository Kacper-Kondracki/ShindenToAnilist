use crate::converter::database;
use crate::converter::searcher::Searcher;
use crate::converter::shinden;
use itertools::Itertools;
use mimalloc::MiMalloc;
use rayon::prelude::*;
use std::fs::File;
use std::hint::black_box;
use std::io::{BufReader, BufWriter};
use std::time::Instant;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;
#[tokio::test]
async fn shinden_test() {
    let start = Instant::now();
    let list = shinden::get(196402).await.unwrap();
    let elapsed = start.elapsed();

    let file = File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open("shinden-test.json")
        .unwrap();

    let mut buf_writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut buf_writer, &list).unwrap();
    println!("Loading Shinden took: {elapsed:.2?}");
    black_box(list);
}

#[tokio::test]
async fn database_test() {
    let start = Instant::now();
    let mut buf_reader = BufReader::new(File::open("anime-offline-database.jsonl").unwrap());

    let db = database::DatabaseRoot::from_reader(&mut buf_reader).unwrap();
    let elapsed = start.elapsed();
    println!("Loading DB took: {elapsed:.2?}");
    black_box(db);
}

#[tokio::test]
async fn searcher_test() {
    let mut db_reader = BufReader::new(File::open("anime-offline-database.jsonl").unwrap());
    let db = database::DatabaseRoot::from_reader(&mut db_reader).unwrap();

    let mut shinden_reader = BufReader::new(File::open("shinden-test.json").unwrap());
    let shinden = serde_json::from_reader::<_, shinden::AnimeList>(&mut shinden_reader).unwrap();
    // Shingeki no Kyojin
    // let shinden_entry = shinden.items.iter().find(|x| x.title_id == 14418).unwrap();

    let searcher = Searcher::new(&db.data);

    let start = Instant::now();
    let matches = shinden
        .items
        .par_iter()
        .map(|entry| (entry, searcher.search(entry.title.as_str(), 50, 0.65)))
        .collect::<Vec<_>>();
    let elapsed = start.elapsed();

    for (entry, results) in matches {
        println!("======== {} ========", entry.title);
        for result in results {
            println!("{:.2} = {}", result.score, result.item.title);
        }
    }

    println!("{:.2?}", elapsed);
}
