use std::{
    fs::File,
    io::BufReader,
    time::Instant,
};

use rayon::prelude::*;
use shinden_to_anilist_core::{
    common::AnimeList,
    database::{
        AnimeDatabase,
        AnimeDatabaseLoad,
    },
    matcher::{
        DefaultMatcher,
        MatchResult,
        Matcher,
        MatcherFinalizer,
    },
    providers::shinden::{
        self,
        ShindenList,
    },
    searcher::{
        DefaultSearcher,
        Search,
        SearcherAnimeExt,
    },
};

fn main() {
    let database = AnimeDatabase::get_from_mmap("anime-offline-database.jsonl").unwrap();
    let shinden: ShindenList =
        serde_json::from_reader(BufReader::new(File::open("shinden-test.json").unwrap())).unwrap();

    let searcher = DefaultSearcher::new(&database);
    let matcher = DefaultMatcher::strict_preset();
    dbg!(matcher);
    let now = Instant::now();

    let mut results: Vec<(&shinden::AnimeEntry, MatchResult)> = shinden
        .par_values()
        .map(|entry| entry.search_by_title_ref(&database, &searcher, Search::options().strict().build()))
        .map(|(entry, candidates)| (entry, matcher.score_candidates(entry, &candidates, 0.5)))
        .collect();

    results.iter_mut().map(|(_, result)| result).finalize_matches();

    let elapsed = now.elapsed();

    for (entry, result) in results.iter() {
        println!("=== {} ===", entry.title());
        for (db_entry, scores) in result.items_ref(&database) {
            println!(
                "[{:.2} {:3}] {}",
                scores.final_score,
                if Some(db_entry.id()) == result.winner().map(|x| x.0) {
                    "WIN"
                } else {
                    ""
                },
                db_entry.title()
            )
        }
    }

    let winners_count = results.iter().filter(|(_, m)| m.winner().is_some()).count();
    let tops_count = results.iter().filter(|(_, m)| !m.top().is_empty()).count();

    println!("TOOK       : {:.2?}", elapsed);
    println!("HAS TOP    : {}/{}", tops_count, shinden.len());
    println!("HAS WINNER : {}/{}", winners_count, shinden.len());
}
