use std::time::Instant;

use futures_util::StreamExt;
use shinden_to_anilist_core::{
    HttpClient,
    common::AnimeList,
    providers::ogladajanime::{
        OgladajAnimeFetchEvent,
        OgladajAnimeList,
        OgladajAnimeListLoad,
    },
};

#[tokio::main]
async fn main() {
    let now = Instant::now();
    let mut entries = Vec::new();
    let client = HttpClient::builder().cookie_store(true).build().unwrap();
    let stream = OgladajAnimeList::stream_from_ogladajanime(client, "746170");

    futures_util::pin_mut!(stream);
    while let Some(event) = stream.next().await {
        match event.unwrap() {
            OgladajAnimeFetchEvent::Started { total_entries } => {
                println!("discovered {total_entries} entries");
            },
            OgladajAnimeFetchEvent::Entry {
                current,
                total_entries,
                entry,
            } => {
                if current == 1 || current % 5 == 0 || current == total_entries {
                    println!("fetched details {current}/{total_entries}");
                }
                entries.push(entry);
            },
        }
    }

    let ogladajanime = OgladajAnimeList::from_entries(entries);
    let elapsed = now.elapsed();

    println!("{} entries", ogladajanime.len());
    println!("{} direct MAL matches", ogladajanime.direct_mal_matches().count());
    println!("{} missing MAL links", ogladajanime.missing_mal_id_count());
    println!("took {:.2?}", elapsed);
}
