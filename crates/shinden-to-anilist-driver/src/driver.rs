use std::sync::{
    Mutex,
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

pub struct StaDriver {
    aborted: AtomicBool,
    database: Mutex<Option<AnimeDatabase>>,
    searcher: Mutex<Option<DefaultSearcher>>,
    shinden_list: Mutex<Option<ShindenList>>,
    match_results: Mutex<Option<Vec<StoredShindenMatchResult>>>,
    shinden_entry_ids: Mutex<StoredShindenEntryIds>,
}

impl StaDriver {
    pub fn new() -> Self {
        Self {
            aborted: AtomicBool::new(false),
            database: Mutex::new(None),
            searcher: Mutex::new(None),
            shinden_list: Mutex::new(None),
            match_results: Mutex::new(None),
            shinden_entry_ids: Mutex::new(StoredShindenEntryIds::default()),
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

    pub(crate) fn database(&self) -> &Mutex<Option<AnimeDatabase>> { &self.database }

    pub(crate) fn searcher(&self) -> &Mutex<Option<DefaultSearcher>> { &self.searcher }

    pub(crate) fn shinden_list(&self) -> &Mutex<Option<ShindenList>> { &self.shinden_list }

    pub(crate) fn match_results(&self) -> &Mutex<Option<Vec<StoredShindenMatchResult>>> {
        &self.match_results
    }

    pub(crate) fn shinden_entry_ids(&self) -> &Mutex<StoredShindenEntryIds> {
        &self.shinden_entry_ids
    }
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
