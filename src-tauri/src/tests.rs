use crate::converter::database;
use crate::converter::regexes;
use crate::converter::searcher::Searcher;
use crate::converter::shinden;
use mimalloc::MiMalloc;
use rayon::prelude::*;
use regex::Regex;
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

    for (entry, results) in &matches {
        println!("======== {} ========", entry.title);
        for result in results {
            println!("{:.2} = {}", result.score, result.item.title);
        }
    }

    let match_count = matches
        .iter()
        .filter(|(_, results)| !results.is_empty())
        .count();

    let strong_count = matches
        .iter()
        .filter(|(_, results)| results.iter().any(|x| x.score >= 0.95))
        .count();

    println!(
        ">=0.95: {}\nFOUND: {}/{}\nNOT FOUND: {}",
        strong_count,
        match_count,
        shinden.items.len(),
        shinden.items.len() - match_count
    );
    println!("{:.2?}", elapsed);
}

#[tokio::test]
async fn regex_test() {
    let regexes = [
        ("YEAR", &*regexes::YEAR),
        ("SEASON_DECIMAL", &*regexes::SEASON_DECIMAL),
        ("SEASON_ROMAN", &*regexes::SEASON_ROMAN),
        ("SEASON_NUMERAL", &*regexes::SEASON_NUMERAL),
        ("DECIMAL_SEASON", &*regexes::DECIMAL_SEASON),
        ("ROMAN_SEASON", &*regexes::ROMAN_SEASON),
        ("NUMERAL_SEASON", &*regexes::NUMERAL_SEASON),
        ("SEASON_DECIMAL_END", &*regexes::SEASON_DECIMAL_END),
        ("SEASON_ROMAN_END", &*regexes::SEASON_ROMAN_END),
        ("SEASON_NUMERAL_END", &*regexes::SEASON_NUMERAL_END),
        ("PART_DECIMAL", &*regexes::PART_DECIMAL),
        ("PART_ROMAN", &*regexes::PART_ROMAN),
        ("PART_NUMERAL", &*regexes::PART_NUMERAL),
        ("DECIMAL_PART", &*regexes::DECIMAL_PART),
        ("ROMAN_PART", &*regexes::ROMAN_PART),
        ("NUMERAL_PART", &*regexes::NUMERAL_PART),
        ("ANIME_TYPE", &*regexes::ANIME_TYPE),
    ];
    fn check_regex(regexes: &[(&str, &Regex)], title: &str) {
        println!("=========\t{title}\t=========\n");

        for regex in regexes {
            let name = regex.0;
            let regex = regex.1;

            let matches = regex
                .captures_iter(title)
                .map(|x| x.get(1).unwrap().as_str())
                .last();

            let Some(matches) = matches else {
                continue;
            };
            let replaced = regex.replace_all(title, "<!!>");
            println!("{name} => {replaced} => ({matches})");
        }
    }

    let mut shinden_reader = BufReader::new(File::open("shinden-test.json").unwrap());
    let shinden = serde_json::from_reader::<_, shinden::AnimeList>(&mut shinden_reader).unwrap();

    let titles = shinden
        .items
        .iter()
        .map(|x| x.title.clone())
        .filter(|x| {
            ["shingeki no", "boku no hero", "jojo"]
                .iter()
                .any(|test| x.to_lowercase().contains(test))
        })
        .collect::<Vec<_>>();

    for title in &titles {
        check_regex(&regexes, title);
    }
}
