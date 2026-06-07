use std::sync::{
    RwLock,
    atomic::{
        AtomicBool,
        Ordering,
    },
};

use shinden_to_anilist_core::{
    database::AnimeDatabase,
    providers::shinden::ShindenList,
    searcher::DefaultSearcher,
};

use crate::ffi::StaScoredCandidate;

#[derive(Clone)]
pub(crate) struct StoredMatchResult {
    pub items: Vec<StaScoredCandidate>,
    pub top: Vec<StaScoredCandidate>,
    pub winner: Option<StaScoredCandidate>,
}

#[derive(Clone)]
pub(crate) struct StoredShindenMatchResult {
    pub shinden_id: u64,
    pub result: StoredMatchResult,
}

#[derive(Clone, Default)]
pub(crate) struct StoredShindenEntryIds {
    pub manual: Vec<u64>,
    pub automatic: Vec<u64>,
    pub all: Vec<u64>,
}

#[derive(Default)]
pub(crate) struct DatabaseState {
    pub generation: u64,
    pub database: Option<AnimeDatabase>,
    pub searcher: Option<DefaultSearcher>,
}

#[derive(Default)]
pub(crate) struct ShindenState {
    pub generation: u64,
    pub list: Option<ShindenList>,
    pub entry_ids: StoredShindenEntryIds,
}

#[derive(Default)]
pub(crate) struct MatchState {
    pub database_generation: u64,
    pub shinden_generation: u64,
    pub results: Option<Vec<StoredShindenMatchResult>>,
}

pub struct StaDriver {
    aborted: AtomicBool,
    database: RwLock<DatabaseState>,
    shinden: RwLock<ShindenState>,
    matches: RwLock<MatchState>,
}

impl StaDriver {
    pub fn new() -> Self {
        Self {
            aborted: AtomicBool::new(false),
            database: RwLock::new(DatabaseState::default()),
            shinden: RwLock::new(ShindenState::default()),
            matches: RwLock::new(MatchState::default()),
        }
    }

    pub(crate) fn abort(&self) { self.aborted.store(true, Ordering::SeqCst); }

    pub(crate) fn is_aborted(&self) -> bool { self.aborted.load(Ordering::SeqCst) }

    pub(crate) fn check_aborted(&self) -> Result<(), String> {
        if self.is_aborted() {
            Err("driver call aborted".to_owned())
        } else {
            Ok(())
        }
    }

    pub(crate) fn database_state(&self) -> &RwLock<DatabaseState> { &self.database }

    pub(crate) fn shinden_state(&self) -> &RwLock<ShindenState> { &self.shinden }

    pub(crate) fn match_state(&self) -> &RwLock<MatchState> { &self.matches }
}

impl Default for StaDriver {
    fn default() -> Self { Self::new() }
}

pub fn new() -> *mut StaDriver { Box::into_raw(Box::new(StaDriver::new())) }

pub fn abort(driver: *mut StaDriver) {
    if let Some(driver) = as_ref(driver) {
        driver.abort();
    }
}

/// # Safety
/// The pointer must have been allocated by [`new`] and must not be used again.
pub unsafe fn free(driver: *mut StaDriver) {
    if !driver.is_null() {
        drop(unsafe { Box::from_raw(driver) });
    }
}

pub(crate) fn as_ref<'a>(driver: *mut StaDriver) -> Option<&'a StaDriver> {
    if driver.is_null() {
        None
    } else {
        Some(unsafe { &*driver })
    }
}
