use std::{
    fs::File,
    io::BufReader,
};

use egobox_ego::EgorBuilder;
use itertools::Itertools;
use ndarray::{
    Array2,
    ArrayView2,
    array,
};
use rayon::prelude::*;
use shinden_to_anilist_core::{
    common::AnimeList,
    database::{
        self,
        AnimeDatabase,
        AnimeDatabaseLoad,
    },
    matcher::{
        DefaultMatcher,
        MatchResult,
        Matcher,
        generate_weights,
    },
    providers::shinden::{
        self,
        ShindenList,
    },
    searcher::{
        DefaultSearcher,
        Search,
        Searcher,
        SearcherAnimeExt,
    },
};

fn main() {
    let database = AnimeDatabase::get_from_mmap("anime-offline-database.jsonl").unwrap();
    let shinden: ShindenList =
        serde_json::from_reader(BufReader::new(File::open("shinden-test.json").unwrap())).unwrap();

    let searcher = DefaultSearcher::new(&database);
    let xlimits = array![
        [0.8, 1.0],
        [0.1, 0.5],
        [0.7, 1.0],
        [0.7, 1.0],
        [0.1, 0.5],
        [0.1, 1.0],
        [0.1, 0.5],
        [0.0, 1.0]
    ];

    let egor = EgorBuilder::optimize(|x: &ArrayView2<f64>| {
        let mut results = Array2::zeros((x.nrows(), 1));

        for i in 0..x.nrows() {
            if let Some(params) = x.row(i).as_slice() {
                results[[i, 0]] = -score_match(params, &shinden, &database, &searcher);
            }
        }

        results
    })
    .configure(|config| config.max_iters(200).trego(true))
    .min_within(&xlimits)
    .unwrap();

    let egor = egor.run().unwrap();

    let best_x = egor.x_opt.as_slice().unwrap();
    let best_y = -egor.y_opt.as_slice().unwrap()[0];

    println!(
        "Best params: [{}]",
        best_x.iter().map(|x| format!("{x:.2}")).join(", ")
    );
    println!("Best score: {:?}/{}", best_y as i64, shinden.len());
}

fn score_match(
    params: &[f64],
    shinden: &impl AnimeList<Entry = shinden::AnimeEntry>,
    database: &impl AnimeList<Entry = database::AnimeEntry>,
    searcher: &(impl Searcher + Sync),
) -> f64 {
    let gamma = params[7];

    let mut weights: Vec<f32> = params[..7].iter().map(|&x| x as f32).collect();

    generate_weights(&mut weights, gamma as f32);

    let matcher = DefaultMatcher::from_weights(*weights.as_array().unwrap(), 0.75, 0.075);

    let results: Vec<MatchResult> = shinden
        .par_values()
        .map(|x| x.search_by_title_ref(database, searcher, Search::options().strict().build()))
        .map(|(entry, cands)| matcher.score_candidates(entry, &cands, 0.5))
        .collect();

    results.iter().filter(|m| m.winner().is_some()).count() as f64
}
