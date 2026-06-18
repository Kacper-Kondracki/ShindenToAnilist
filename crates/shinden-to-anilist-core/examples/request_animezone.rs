use std::time::Instant;

use shinden_to_anilist_core::{
    HttpClient,
    common::AnimeList,
    providers::animezone::{
        AnimeZoneList,
        AnimeZoneListLoad,
    },
};

#[tokio::main]
async fn main() {
    let now = Instant::now();
    let animezone = AnimeZoneList::get_from_animezone(HttpClient::new(), "JestemOtaku")
        .await
        .unwrap();
    let elapsed = now.elapsed();

    println!("{} entries", animezone.len());
    println!("{} direct MAL matches", animezone.direct_mal_matches().count());
    println!("{} missing MAL links", animezone.missing_mal_id_count());
    println!("took {:.2?}", elapsed);
}
