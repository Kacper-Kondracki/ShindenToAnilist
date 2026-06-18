use std::{
    collections::HashSet,
    time::Instant,
};

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
    providers::animezone::{
        AnimeZoneEntry,
        AnimeZoneList,
        AnimeZoneListLoad,
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
    let animezone = AnimeZoneList::get_from_animezone(HttpClient::new(), "JestemOtaku")
        .await
        .unwrap();

    let direct_matches = animezone
        .direct_mal_matches()
        .filter(|(_, mal_id)| database.get(*mal_id).is_some())
        .collect::<Vec<_>>();
    let direct_entry_ids = direct_matches
        .iter()
        .map(|(entry_id, _)| *entry_id)
        .collect::<HashSet<_>>();

    let mut fallback_results = animezone
        .entries_without_mal_id()
        .map(|entry| entry.search_by_title_ref(&database, &searcher, Search::options().strict().build()))
        .map(|(entry, candidates)| (entry, matcher.score_candidates(entry, &candidates, 0.5)))
        .collect::<Vec<(&AnimeZoneEntry, _)>>();

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
    let unmatched = animezone
        .keys()
        .filter(|id| !direct_entry_ids.contains(id) && !fallback_entry_ids.contains(id))
        .count();

    println!("{} entries", animezone.len());
    println!("{} direct matches", direct_matches.len());
    println!("{} fallback matches", fallback_matches.len());
    println!("{} unmatched", unmatched);
    println!("took {:.2?}", now.elapsed());

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
}
