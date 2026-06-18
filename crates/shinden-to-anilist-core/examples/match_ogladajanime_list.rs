use std::{
    collections::HashSet,
    time::Instant,
};

use futures_util::StreamExt;
use shinden_to_anilist_core::{
    HttpClient,
    common::AnimeList,
    database::{
        AnimeDatabase,
        AnimeDatabaseLoad,
    },
    matcher::{
        DefaultMatcher,
        Matcher,
        MatcherFinalizer,
    },
    providers::ogladajanime::{
        OgladajAnimeEntry,
        OgladajAnimeFetchEvent,
        OgladajAnimeList,
        OgladajAnimeListLoad,
    },
    searcher::{
        DefaultSearcher,
        Search,
        SearcherAnimeExt,
    },
};

#[tokio::main]
async fn main() {
    let database = AnimeDatabase::get_from_mmap("anime-offline-database.jsonl").unwrap();
    let searcher = DefaultSearcher::new(&database);
    let matcher = DefaultMatcher::strict_preset();

    let now = Instant::now();
    let mut entries = Vec::new();
    let client = HttpClient::builder().cookie_store(true).build().unwrap();
    let stream = OgladajAnimeList::stream_from_ogladajanime(client, "746170");

    futures_util::pin_mut!(stream);
    while let Some(event) = stream.next().await {
        match event.unwrap() {
            OgladajAnimeFetchEvent::Started { total_entries } => {
                println!("discovered {total_entries} OgladajAnime entries");
            },
            OgladajAnimeFetchEvent::Entry {
                current,
                total_entries,
                entry,
            } => {
                if current == 1 || current % 50 == 0 || current == total_entries {
                    println!("fetched OgladajAnime details {current}/{total_entries}");
                }
                entries.push(entry);
            },
        }
    }

    let ogladajanime = OgladajAnimeList::from_entries(entries);

    let direct_matches = ogladajanime
        .direct_mal_matches()
        .filter(|(_, mal_id)| database.get(*mal_id).is_some())
        .collect::<Vec<_>>();
    let direct_entry_ids = direct_matches
        .iter()
        .map(|(entry_id, _)| *entry_id)
        .collect::<HashSet<_>>();

    let mut fallback_results = ogladajanime
        .entries_without_mal_id()
        .map(|entry| entry.search_by_title_ref(&database, &searcher, Search::options().strict().build()))
        .map(|(entry, candidates)| (entry, matcher.score_candidates(entry, &candidates, 0.5)))
        .collect::<Vec<(&OgladajAnimeEntry, _)>>();

    fallback_results
        .iter_mut()
        .map(|(_, result)| result)
        .finalize_matches();

    let fallback_matches = fallback_results
        .iter()
        .filter_map(|(entry, result)| result.winner().map(|(database_id, _)| (entry.id(), database_id)))
        .collect::<Vec<_>>();
    let fallback_entry_ids = fallback_matches
        .iter()
        .map(|(entry_id, _)| *entry_id)
        .collect::<HashSet<_>>();
    let unmatched_entries = ogladajanime
        .iter()
        .map(|(_, entry)| entry)
        .filter(|&entry| !direct_entry_ids.contains(&entry.id()) && !fallback_entry_ids.contains(&entry.id()))
        .collect::<Vec<_>>();

    println!("{} entries", ogladajanime.len());
    println!("{} direct matches", direct_matches.len());
    println!("{} fallback matches", fallback_matches.len());
    println!("{} unmatched", unmatched_entries.len());
    println!("took {:.2?}", now.elapsed());

    println!();
    println!("fallback matches:");
    for (entry, result) in fallback_results.iter().take(10) {
        if let Some((database_id, score)) = result.winner() {
            let database_entry = database.get_unwrap(database_id);
            println!(
                "[fallback {:.2}] {} -> {}",
                score.final_score,
                entry.title(),
                database_entry.title()
            );
        }
    }

    println!();
    println!("unmatched entries:");
    for entry in unmatched_entries {
        println!("[unmatched] {} ({})", entry.title(), entry.id());
    }
}
