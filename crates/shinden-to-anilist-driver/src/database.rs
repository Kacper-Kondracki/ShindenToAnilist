use std::path::Path;

use shinden_to_anilist_core::{
    BlockingHttpClient,
    common::AnimeList,
    database::{
        AnimeDatabase,
        AnimeDatabaseLoad,
        AnimeEntry,
        root_metadata_from_path,
        updater::{
            DatabaseUpdateStatus,
            update_latest_jsonl_from_github_blocking,
        },
    },
    extractor::{
        ConsolidatedMetadata,
        TitleMetadata,
    },
    searcher::DefaultSearcher,
};

use crate::{
    driver::StaDriver,
    ffi::{
        StaAnimeDatabase,
        StaConsolidatedMetadata,
        StaDatabaseEntry,
        StaDatabaseInfo,
        StaTitleMetadata,
        into_raw_string,
        optional_date,
        optional_f32,
        optional_i32,
        string_view,
        string_view_array,
    },
    labels,
};

pub fn ensure_database(driver: &StaDriver, path: &str) -> Result<StaDatabaseInfo, String> {
    driver.check_aborted()?;

    let update_status = update_latest_jsonl_from_github_blocking(BlockingHttpClient::new(), path)
        .map_err(|error| error.to_string())?;
    driver.check_aborted()?;

    let metadata = root_metadata_from_path(path).map_err(|error| error.to_string())?;
    let database = AnimeDatabase::get_from_mmap(path).map_err(|error| error.to_string())?;
    let searcher = DefaultSearcher::new(&database);
    driver.check_aborted()?;

    let mut state = driver
        .database_state()
        .write()
        .map_err(|_| "database state lock is poisoned".to_owned())?;
    state.generation = state.generation.wrapping_add(1);
    state.database = Some(database);
    state.searcher = Some(searcher);
    drop(state);

    let mut matches = driver
        .match_state()
        .write()
        .map_err(|_| "match state lock is poisoned".to_owned())?;
    matches.results = None;
    drop(matches);

    let mut shinden = driver
        .shinden_state()
        .write()
        .map_err(|_| "shinden state lock is poisoned".to_owned())?;
    shinden.entry_ids.automatic = Vec::new();
    shinden.entry_ids.manual = shinden.entry_ids.all.clone();

    let (release, sha256, updated) = match update_status {
        DatabaseUpdateStatus::UpToDate { release, sha256 } => (release, sha256, false),
        DatabaseUpdateStatus::Updated { release, sha256, .. } => (release, sha256, true),
    };

    Ok(StaDatabaseInfo {
        last_update: into_raw_string(metadata.last_update().to_string()),
        release: into_raw_string(release),
        sha256: into_raw_string(sha256),
        path: into_raw_string(Path::new(path).display().to_string()),
        updated,
    })
}

pub fn get_database_entries(driver: &StaDriver, ids: &[u64]) -> Result<StaAnimeDatabase, String> {
    driver.check_aborted()?;

    let state = driver
        .database_state()
        .read()
        .map_err(|_| "database state lock is poisoned".to_owned())?;
    let database = state
        .database
        .as_ref()
        .ok_or_else(|| "anime database is not loaded".to_owned())?;

    let mut entries = Vec::with_capacity(ids.len());
    for id in ids {
        let entry = database
            .get(*id)
            .ok_or_else(|| format!("database entry {id} is not loaded"))?;
        entries.push(entry_to_ffi(entry));
    }

    driver.check_aborted()?;
    entries.shrink_to_fit();
    let len = entries.len();
    let entries = entries.leak().as_mut_ptr();

    Ok(StaAnimeDatabase {
        last_update: optional_date(Some(database.last_update())),
        entries,
        len,
    })
}

fn entry_to_ffi(entry: &AnimeEntry) -> StaDatabaseEntry {
    StaDatabaseEntry {
        id: entry.id(),
        consolidated_metadata: consolidated_metadata_to_ffi(entry.consolidated_metadata()),
        sources: string_view_array(entry.sources().iter().map(|value| value.as_str())),
        title: string_view(entry.title()),
        normalized_title: string_view(entry.normalized_title()),
        metadata: title_metadata_to_ffi(entry.metadata()),
        anime_type: string_view(labels::anime_type(entry.anime_type())),
        episodes: entry.episodes(),
        status: string_view(labels::anime_status(entry.status())),
        season: string_view(labels::season(entry.season())),
        year: optional_i32(entry.year()),
        picture: string_view(entry.picture()),
        thumbnail: string_view(entry.thumbnail()),
        duration: optional_i32(entry.duration()),
        synonyms: string_view_array(entry.synonyms().iter().map(|value| value.as_str())),
        normalized_synonyms: string_view_array(
            entry.normalized_synonyms().iter().map(|value| value.as_str()),
        ),
        studios: string_view_array(entry.studios().iter().map(|value| value.as_str())),
        producers: string_view_array(entry.producers().iter().map(|value| value.as_str())),
        related_anime: string_view_array(entry.related_anime().iter().map(|value| value.as_str())),
        tags: string_view_array(entry.tags().iter().map(|value| value.as_str())),
    }
}

pub(crate) fn title_metadata_to_ffi(value: &TitleMetadata) -> StaTitleMetadata {
    StaTitleMetadata {
        season: optional_f32(value.season()),
        part: optional_f32(value.part()),
        episode: optional_f32(value.episode()),
        has_season_keyword: value.has_season_keyword(),
        has_part_keyword: value.has_part_keyword(),
        has_episode_keyword: value.has_episode_keyword(),
    }
}

fn consolidated_metadata_to_ffi(value: ConsolidatedMetadata) -> StaConsolidatedMetadata {
    StaConsolidatedMetadata {
        season: optional_f32(value.season()),
        part: optional_f32(value.part()),
        episode: optional_f32(value.episode()),
        is_final_season: value.is_final_season(),
        is_final_part: value.is_final_part(),
        is_final_episode: value.is_final_episode(),
    }
}
