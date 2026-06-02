use std::{
    fs::File,
    io::BufReader,
    time::Instant,
};

use rayon::prelude::*;
use shinden_to_anilist_core::{
    common::{
        AnimeId,
        AnimeList,
    },
    database::{
        AnimeDatabase,
        AnimeDatabaseLoad,
    },
    exporter::{
        ExportExt,
        xml::XmlExporter,
    },
    matcher::{
        DefaultMatcher,
        MatchResult,
        Matcher,
        MatcherFinalizer,
    },
    providers::shinden::ShindenList,
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

    let mut results: Vec<(AnimeId, MatchResult)> = shinden
        .par_values()
        .map(|entry| entry.search_by_title_ref(&database, &searcher, Search::options().strict().build()))
        .map(|(entry, candidates)| (entry.id(), matcher.score_candidates(entry, &candidates, 0.5)))
        .collect();
    results.iter_mut().map(|(_, result)| result).finalize_matches();

    let results: Vec<(AnimeId, AnimeId)> = results
        .into_iter()
        .filter_map(|(id, result)| result.winner().map(|(winner, _)| (id, winner)))
        .step_by(100)
        .collect();

    let now = Instant::now();
    let exporter = XmlExporter {};
    shinden
        .export(
            &exporter,
            results.iter().copied(),
            File::options()
                .create(true)
                .truncate(true)
                .write(true)
                .open("xml-test.xml")
                .unwrap(),
        )
        .unwrap();
    let elapsed = now.elapsed();
    println!("took {:.2?}", elapsed);
}
