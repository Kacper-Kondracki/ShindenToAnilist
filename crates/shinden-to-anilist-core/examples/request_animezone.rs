use std::time::Instant;

use futures_util::StreamExt;
use shinden_to_anilist_core::{
    HttpClient,
    common::AnimeList,
    providers::animezone::{
        AnimeZoneFetchEvent,
        AnimeZoneList,
        AnimeZoneListLoad,
    },
};

#[tokio::main]
async fn main() {
    let now = Instant::now();
    let mut entries = Vec::new();
    let stream = AnimeZoneList::stream_from_animezone(HttpClient::new(), "JestemOtaku");

    futures_util::pin_mut!(stream);
    while let Some(event) = stream.next().await {
        match event.unwrap() {
            AnimeZoneFetchEvent::Started { total_entries } => {
                println!("discovered {total_entries} entries");
            },
            AnimeZoneFetchEvent::Entry {
                current,
                total_entries,
                entry,
            } => {
                if current == 1 || current % 50 == 0 || current == total_entries {
                    println!("fetched details {current}/{total_entries}");
                }
                entries.push(entry);
            },
        }
    }

    let animezone = AnimeZoneList::from_entries(entries);
    let elapsed = now.elapsed();

    println!("{} entries", animezone.len());
    println!("{} direct MAL matches", animezone.direct_mal_matches().count());
    println!("{} missing MAL links", animezone.missing_mal_id_count());
    println!("took {:.2?}", elapsed);
}
