use shinden_to_anilist_core::{
    BlockingHttpClient,
    common::AnimeList,
    providers::shinden::{
        ShindenList,
        ShindenListLoad,
    },
};

use crate::{
    driver::StaDriver,
    ffi::{
        StaShindenEntry,
        StaShindenList,
        optional_date,
        optional_i32,
        optional_string_view,
        string_view,
    },
    labels,
};

pub fn load_list(driver: &StaDriver, user_id: u64) -> Result<StaShindenList, String> {
    let list = ShindenList::get_from_shinden_blocking(BlockingHttpClient::new(), user_id)
        .map_err(|error| error.to_string())?;

    let mut shinden_list = driver
        .shinden_list()
        .lock()
        .map_err(|_| "shinden list lock is poisoned".to_owned())?;
    *shinden_list = Some(list);

    let list = shinden_list
        .as_ref()
        .ok_or_else(|| "loaded shinden list is unavailable".to_owned())?;

    let mut entries = list.values().map(entry_to_ffi).collect::<Vec<_>>();

    entries.shrink_to_fit();
    let len = entries.len();
    let entries = entries.leak().as_mut_ptr();

    Ok(StaShindenList { entries, len })
}

fn entry_to_ffi(entry: &<ShindenList as AnimeList>::Entry) -> StaShindenEntry {
    StaShindenEntry {
        id: entry.id(),
        cover_id: optional_i32(entry.cover_id()),
        title: string_view(entry.title()),
        anime_status: string_view(labels::anime_status(entry.anime_status())),
        anime_type: string_view(labels::anime_type(entry.anime_type())),
        premiere_date: optional_date(entry.premiere_date()),
        finish_date: optional_date(entry.finish_date()),
        episodes: optional_i32(entry.episodes()),
        is_favourite: entry.is_favourite(),
        watch_status: string_view(labels::watch_status(entry.watch_status())),
        watched_episodes: entry.watched_episodes(),
        score: optional_i32(entry.score()),
        note: optional_string_view(entry.note().map(|value| value.as_str())),
        description: optional_string_view(entry.description().map(|value| value.as_str())),
    }
}
