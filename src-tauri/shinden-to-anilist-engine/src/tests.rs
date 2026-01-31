use crate::{
    converter::database, converter::matcher::*, converter::regexes, converter::searcher::Searcher,
    converter::shinden, converter::shinden::ShindenList,
};
use egobox_ego::EgorBuilder;
use indexmap::IndexMap;
use itertools::Itertools;
use mimalloc::MiMalloc;
use ndarray::{Array2, ArrayView2, array};
use owo_colors::OwoColorize;
use rayon::prelude::*;
use regex::Regex;
use serde::{Serialize, de::DeserializeOwned};
use std::{
    fs,
    fs::File,
    hint::black_box,
    io::Write,
    io::{BufReader, BufWriter},
    iter,
    path::Path,
    sync::Arc,
    time::Instant,
};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn serialize_to_file(path: impl AsRef<Path>, obj: &impl Serialize) {
    let file = File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .unwrap();
    let mut buf_writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut buf_writer, obj).unwrap();
}

fn deserialize_from_file<T: DeserializeOwned>(path: impl AsRef<Path>) -> T {
    let mut buf_reader = BufReader::new(File::open(path).unwrap());
    serde_json::from_reader(&mut buf_reader).unwrap()
}

#[tokio::test]
async fn shinden_test() {
    let start = Instant::now();
    // 196402
    let list = shinden::get(300263).await.unwrap();
    let elapsed = start.elapsed();
    serialize_to_file("shinden-test.json", &list);
    println!("Loading Shinden took: {elapsed:.2?}");
    black_box(list);
}

#[tokio::test]
async fn database_test() {
    let start = Instant::now();
    let db = database::DatabaseRoot::from_path("anime-offline-database.jsonl").unwrap();
    let elapsed = start.elapsed();
    println!("Loading DB took: {elapsed:.2?}");
    black_box(db);
}

#[tokio::test]
async fn searcher_test() {
    let db = Arc::new(database::DatabaseRoot::from_path("anime-offline-database.jsonl").unwrap());
    let shinden = deserialize_from_file::<ShindenList>("shinden-test.json");

    let start = Instant::now();
    let searcher = Searcher::new(db.clone());
    let init_elapsed = start.elapsed();

    let start = Instant::now();
    let matches = shinden
        .items
        .par_iter()
        .map(|(&id, entry)| {
            (
                id,
                (
                    entry,
                    searcher.search(entry.title.as_str(), 50, 0.65, false),
                ),
            )
        })
        .collect::<IndexMap<_, _>>();
    let search_elapsed = start.elapsed();

    for (_, (entry, results)) in &matches {
        println!("======== {} ========", entry.title);
        for result in results {
            println!("{:.2} = {}", result.score, result.item.title);
        }
    }

    let match_count = matches
        .iter()
        .filter(|(_, (_, results))| !results.is_empty())
        .count();

    let strong_count = matches
        .iter()
        .filter(|(_, (_, results))| results.iter().any(|x| x.score >= 0.95))
        .count();

    println!(
        ">=0.95: {}\nFOUND: {}/{}\nNOT FOUND: {}",
        strong_count,
        match_count,
        shinden.items.len(),
        shinden.items.len() - match_count
    );
    println!("Init: {:.2?}\nSearch: {:.2?}", init_elapsed, search_elapsed);
}

#[tokio::test]
async fn matcher_test() {
    let db = Arc::new(database::DatabaseRoot::from_path("anime-offline-database.jsonl").unwrap());
    let shinden = deserialize_from_file::<ShindenList>("shinden-test.json");

    let searcher = Searcher::new(db.clone());

    let start = Instant::now();
    let matches = shinden
        .items
        .par_iter()
        .map(|(&id, entry)| {
            (
                id,
                (
                    entry,
                    searcher.search_shinden(entry, 50, 0.65, true, MatcherConfig::and_preset()),
                ),
            )
        })
        .collect::<IndexMap<_, _>>();
    let match_elapsed = start.elapsed();

    for (_, (entry, results)) in &matches {
        println!("======== {} ========", entry.title);
        for result in results {
            let color = if result.likely_match {
                owo_colors::AnsiColors::Green
            } else {
                owo_colors::AnsiColors::Yellow
            };
            println!(
                "{:.2} = {}",
                result.score_breakdown.final_score.color(color),
                result.candidate.title.color(color)
            );
        }
    }

    let single_matches_count = matches
        .iter()
        .filter(|(_, (_, cands))| cands.iter().filter(|x| x.likely_match).count() == 1)
        .count();

    let strong_matches_count = matches
        .iter()
        .filter(|(_, (_, cands))| cands.iter().filter(|x| x.likely_match).count() > 0)
        .count();

    println!(
        "STRONG MATCHES: {}/{}",
        strong_matches_count,
        shinden.items.len(),
    );

    println!(
        "SINGLE MATCHES: {}/{}",
        single_matches_count,
        shinden.items.len(),
    );

    println!("Match: {:.2?}", match_elapsed);
}

fn matcher_objective(
    x: &ArrayView2<f64>,
    shinden: &ShindenList,
    searcher: &Searcher,
    use_and: bool,
) -> Array2<f64> {
    let n = x.nrows();
    let mut y = Array2::<f64>::zeros((n, 1));
    let scores = (0..n)
        .par_bridge()
        .map(|i| {
            let params = x.row(i);

            let weights = &mut params.to_vec()[..8];

            let sum: f64 = weights.iter().sum();

            if sum > 0.0 {
                weights.iter_mut().for_each(|x| *x /= sum);
            } else {
                weights.iter_mut().for_each(|x| *x = 1.0 / 8.0);
            }

            let config = MatcherConfig {
                ngram_weight: weights[0] as f32,
                sim_weight: weights[1] as f32,
                season_part_weight: weights[2] as f32,
                year_weight: weights[3] as f32,
                type_weight: weights[4] as f32,
                status_weight: weights[5] as f32,
                month_season_weight: weights[6] as f32,
                episode_weight: weights[7] as f32,
                match_threshold: params[8] as f32,
                ..Default::default()
            };

            let matches = shinden
                .items
                .par_iter()
                .map(|(&id, entry)| {
                    (
                        id,
                        (
                            entry,
                            searcher.search_shinden(entry, 100, 0.60, use_and, config),
                        ),
                    )
                })
                .collect::<IndexMap<_, _>>();

            let single_matches_count = matches
                .par_iter()
                .filter(|(_, (_, cands))| cands.iter().filter(|x| x.likely_match).count() == 1)
                .count();

            let score: f64 = single_matches_count as f64 / matches.len() as f64;
            -score
        })
        .collect::<Vec<_>>();
    for i in 0..n {
        y[[i, 0]] = scores[i];
    }
    y
}

fn run_optimize(
    shinden: &ShindenList,
    searcher: &Searcher,
    use_and: bool,
    previous: usize,
    limit: usize,
) {
    let bounds = array![
        [0.00, 1.00],
        [0.00, 1.00],
        [0.00, 1.00],
        [0.00, 1.00],
        [0.00, 1.00],
        [0.00, 1.00],
        [0.00, 1.00],
        [0.00, 1.00],
        [0.75, 1.0]
    ];
    let result = EgorBuilder::optimize(|x| matcher_objective(x, shinden, searcher, use_and))
        .configure(|cfg| cfg.n_doe(0).max_iters(limit))
        .min_within(&bounds)
        .expect("invalid config")
        .run()
        .expect("optimization failed");

    let params = result.x_opt.to_vec();
    let weights = &params[..8];
    let sum: f64 = weights.iter().sum();

    let normalized_weights = if sum > 0.0 {
        weights.iter().map(|x| x / sum).collect::<Vec<_>>()
    } else {
        weights.iter().map(|_| 1.0 / 8.0).collect::<Vec<_>>()
    };
    let score = result.y_opt[0];
    let score = (-score * shinden.items.len() as f64) as usize;

    if score > previous {
        let mut out = BufWriter::new(
            File::options()
                .write(true)
                .create(true)
                .truncate(true)
                .open(if use_and {
                    "matcher-opt-and.txt"
                } else {
                    "matcher-opt-or.txt"
                })
                .unwrap(),
        );
        writeln!(out, "{}/{}", score, shinden.items.len()).unwrap();
        writeln!(
            out,
            "[{}]",
            normalized_weights
                .iter()
                .chain(iter::once(params.last().unwrap()))
                .map(|x| format!("{:.2}", x))
                .join(", "),
        )
        .unwrap();
    }
}

fn extract_score_from_file(path: impl AsRef<Path>) -> usize {
    fs::read_to_string(path)
        .ok()
        .and_then(|content| {
            content
                .lines()
                .next()
                .and_then(|line| line.split("/").next().and_then(|t| t.parse::<usize>().ok()))
        })
        .unwrap_or_default()
}

#[tokio::test]
async fn matcher_optimize_and() {
    let db = Arc::new(database::DatabaseRoot::from_path("anime-offline-database.jsonl").unwrap());
    let shinden = deserialize_from_file::<ShindenList>("shinden-test.json");
    let searcher = Searcher::new(db.clone());

    for _ in 0..20 {
        let previous = extract_score_from_file("matcher-opt-and.txt");
        run_optimize(&shinden, &searcher, true, previous, 125);
    }
}

#[tokio::test]
async fn matcher_optimize_or() {
    let db = Arc::new(database::DatabaseRoot::from_path("anime-offline-database.jsonl").unwrap());
    let shinden = deserialize_from_file::<ShindenList>("shinden-test.json");
    let searcher = Searcher::new(db.clone());

    for _ in 0..20 {
        let previous = extract_score_from_file("matcher-opt-or.txt");
        run_optimize(&shinden, &searcher, false, previous, 150);
    }
}

#[tokio::test]
async fn regex_test() {
    let regexes: &[(&str, &Regex)] = &[
        ("YEAR", &regexes::YEAR),
        ("SEASON_DECIMAL", &regexes::SEASON_DECIMAL),
        ("SEASON_ROMAN", &regexes::SEASON_ROMAN),
        ("SEASON_NUMERAL", &regexes::SEASON_NUMERAL),
        ("DECIMAL_SEASON", &regexes::DECIMAL_SEASON),
        ("ROMAN_SEASON", &regexes::ROMAN_SEASON),
        ("NUMERAL_SEASON", &regexes::NUMERAL_SEASON),
        ("SEASON_DECIMAL_END", &regexes::SEASON_DECIMAL_END),
        ("SEASON_ROMAN_END", &regexes::SEASON_ROMAN_END),
        ("SEASON_NUMERAL_END", &regexes::SEASON_NUMERAL_END),
        ("PART_DECIMAL", &regexes::PART_DECIMAL),
        ("PART_ROMAN", &regexes::PART_ROMAN),
        ("PART_NUMERAL", &regexes::PART_NUMERAL),
        ("DECIMAL_PART", &regexes::DECIMAL_PART),
        ("ROMAN_PART", &regexes::ROMAN_PART),
        ("NUMERAL_PART", &regexes::NUMERAL_PART),
        ("ANIME_TYPE", &regexes::ANIME_TYPE),
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

    let shinden = deserialize_from_file::<ShindenList>("shinden-test.json");

    let titles = shinden
        .items
        .iter()
        .map(|(_, x)| x.title.clone())
        .filter(|x| {
            ["shingeki no", "boku no hero", "jojo"]
                .iter()
                .any(|test| x.to_lowercase().contains(test))
        })
        .collect::<Vec<_>>();

    for title in &titles {
        check_regex(regexes, title);
    }
}
