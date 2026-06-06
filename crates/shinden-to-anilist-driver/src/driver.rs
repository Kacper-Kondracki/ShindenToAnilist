use std::sync::Mutex;

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

pub struct StaDriver {
    database: Mutex<Option<AnimeDatabase>>,
    searcher: Mutex<Option<DefaultSearcher>>,
    shinden_list: Mutex<Option<ShindenList>>,
    match_results: Mutex<Option<Vec<StoredShindenMatchResult>>>,
}

impl StaDriver {
    pub fn new() -> Self {
        Self {
            database: Mutex::new(None),
            searcher: Mutex::new(None),
            shinden_list: Mutex::new(None),
            match_results: Mutex::new(None),
        }
    }

    pub(crate) fn database(&self) -> &Mutex<Option<AnimeDatabase>> { &self.database }

    pub(crate) fn searcher(&self) -> &Mutex<Option<DefaultSearcher>> { &self.searcher }

    pub(crate) fn shinden_list(&self) -> &Mutex<Option<ShindenList>> { &self.shinden_list }

    pub(crate) fn match_results(&self) -> &Mutex<Option<Vec<StoredShindenMatchResult>>> {
        &self.match_results
    }
}

impl Default for StaDriver {
    fn default() -> Self { Self::new() }
}

pub fn new() -> *mut StaDriver { Box::into_raw(Box::new(StaDriver::new())) }

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
