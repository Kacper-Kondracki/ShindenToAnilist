use shinden_to_anilist_core::{
    BlockingHttpClient,
    common::AnimeList,
    providers::shinden::{
        ShindenList,
        ShindenListLoad,
    },
};

use crate::{
    driver::{
        StaDriver,
        StoredShindenEntryIds,
    },
    ffi::{
        StaIdList,
        StaShindenEntry,
        StaShindenList,
        optional_date,
        optional_i32,
        optional_string_view,
        string_view,
    },
    labels,
};

pub fn load_list(driver: &StaDriver, user_id: u64) -> Result<StaIdList, String> {
    driver.check_aborted()?;

    let list = ShindenList::get_from_shinden_blocking(BlockingHttpClient::new(), user_id)
        .map_err(|error| error.to_string())?;
    driver.check_aborted()?;

    let sorted_ids = sorted_entry_ids(&list);

    {
        let mut shinden = driver
            .shinden_state()
            .write()
            .map_err(|_| "shinden state lock is poisoned".to_owned())?;
        shinden.generation = shinden.generation.wrapping_add(1);
        shinden.list = Some(list);
        shinden.entry_ids = StoredShindenEntryIds {
            manual: sorted_ids.clone(),
            automatic: Vec::new(),
            all: sorted_ids.clone(),
        };
    }
    {
        let mut matches = driver
            .match_state()
            .write()
            .map_err(|_| "match state lock is poisoned".to_owned())?;
        matches.results = None;
    }

    id_list_to_ffi(sorted_ids)
}

pub fn get_entry_ids(driver: &StaDriver, view: &str) -> Result<StaIdList, String> {
    driver.check_aborted()?;

    let shinden = driver
        .shinden_state()
        .read()
        .map_err(|_| "shinden state lock is poisoned".to_owned())?;
    let ids = match view {
        "" | "manual" => &shinden.entry_ids.manual,
        "automatic" => &shinden.entry_ids.automatic,
        "all" => &shinden.entry_ids.all,
        _ => return Err(format!("unknown shinden entry id view: {view}")),
    };

    id_list_to_ffi(ids.clone())
}

pub fn get_entries(driver: &StaDriver, ids: &[u64]) -> Result<StaShindenList, String> {
    driver.check_aborted()?;

    let shinden = driver
        .shinden_state()
        .read()
        .map_err(|_| "shinden state lock is poisoned".to_owned())?;
    let list = shinden
        .list
        .as_ref()
        .ok_or_else(|| "shinden list is not loaded".to_owned())?;

    let mut entries = Vec::with_capacity(ids.len());
    for id in ids {
        let entry = list
            .get(*id)
            .ok_or_else(|| format!("shinden entry {id} is not loaded"))?;
        entries.push(entry_to_ffi(entry));
    }

    entries.shrink_to_fit();
    let len = entries.len();
    let entries = entries.leak().as_mut_ptr();

    Ok(StaShindenList { entries, len })
}

pub(crate) fn sorted_entry_ids(list: &ShindenList) -> Vec<u64> {
    let mut ids = list.keys().collect::<Vec<_>>();
    ids.sort_by(|left_id, right_id| {
        let left = list.get_unwrap(*left_id);
        let right = list.get_unwrap(*right_id);

        match (left.premiere_date(), right.premiere_date()) {
            (Some(left_date), Some(right_date)) => right_date.cmp(&left_date),
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, None) => std::cmp::Ordering::Equal,
        }
    });
    ids
}

pub(crate) fn id_list_to_ffi(mut ids: Vec<u64>) -> Result<StaIdList, String> {
    ids.shrink_to_fit();
    let len = ids.len();
    let entries = ids.leak().as_mut_ptr();

    Ok(StaIdList { entries, len })
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
