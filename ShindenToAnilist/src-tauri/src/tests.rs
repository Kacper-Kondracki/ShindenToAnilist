use crate::converter::database::models::DatabaseRoot;
use crate::converter::shinden;
use mimalloc::MiMalloc;
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

    let db = DatabaseRoot::from_reader(&mut buf_reader).unwrap();
    let elapsed = start.elapsed();
    println!("Loading DB took: {elapsed:.2?}");
    black_box(db);
}
