use std::{
    ffi::{
        CString,
        c_char,
    },
    panic::{
        AssertUnwindSafe,
        catch_unwind,
    },
    ptr,
};

use shinden_to_anilist_core::{
    Datelike,
    NaiveDate,
};

use crate::driver::{
    self,
    StaDriver,
};

#[repr(C)]
pub enum StaStatus {
    StaStatusOk = 0,
    StaStatusNullPointer = 1,
    StaStatusPanic = 2,
    StaStatusError = 3,
}

#[repr(C)]
pub struct StaError {
    pub status: StaStatus,
    pub message: *mut c_char,
}

#[repr(C)]
pub struct StaDatabaseInfo {
    pub last_update: *mut c_char,
    pub release: *mut c_char,
    pub sha256: *mut c_char,
    pub path: *mut c_char,
    pub updated: bool,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct StaStringView {
    pub ptr: *const c_char,
    pub len: usize,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct StaOptionalI32 {
    pub value: i32,
    pub has_value: bool,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct StaOptionalDate {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub has_value: bool,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct StaOptionalF32 {
    pub value: f32,
    pub has_value: bool,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct StaStringViewArray {
    pub entries: *mut StaStringView,
    pub len: usize,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct StaTitleMetadata {
    pub season: StaOptionalF32,
    pub part: StaOptionalF32,
    pub episode: StaOptionalF32,
    pub has_season_keyword: bool,
    pub has_part_keyword: bool,
    pub has_episode_keyword: bool,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct StaConsolidatedMetadata {
    pub season: StaOptionalF32,
    pub part: StaOptionalF32,
    pub episode: StaOptionalF32,
    pub is_final_season: bool,
    pub is_final_part: bool,
    pub is_final_episode: bool,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct StaDatabaseEntry {
    pub id: u64,
    pub consolidated_metadata: StaConsolidatedMetadata,
    pub sources: StaStringViewArray,
    pub title: StaStringView,
    pub normalized_title: StaStringView,
    pub metadata: StaTitleMetadata,
    pub anime_type: StaStringView,
    pub episodes: i32,
    pub status: StaStringView,
    pub season: StaStringView,
    pub year: StaOptionalI32,
    pub picture: StaStringView,
    pub thumbnail: StaStringView,
    pub duration: StaOptionalI32,
    pub synonyms: StaStringViewArray,
    pub normalized_synonyms: StaStringViewArray,
    pub studios: StaStringViewArray,
    pub producers: StaStringViewArray,
    pub related_anime: StaStringViewArray,
    pub tags: StaStringViewArray,
}

#[repr(C)]
pub struct StaAnimeDatabase {
    pub last_update: StaOptionalDate,
    pub entries: *mut StaDatabaseEntry,
    pub len: usize,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct StaShindenEntry {
    pub id: u64,
    pub cover_id: StaOptionalI32,
    pub title: StaStringView,
    pub anime_status: StaStringView,
    pub anime_type: StaStringView,
    pub premiere_date: StaOptionalDate,
    pub finish_date: StaOptionalDate,
    pub episodes: StaOptionalI32,
    pub is_favourite: bool,
    pub watch_status: StaStringView,
    pub watched_episodes: i32,
    pub score: StaOptionalI32,
    pub note: StaStringView,
    pub description: StaStringView,
}

#[repr(C)]
pub struct StaShindenList {
    pub entries: *mut StaShindenEntry,
    pub len: usize,
}

#[repr(C)]
pub struct StaIdList {
    pub entries: *mut u64,
    pub len: usize,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct StaSearchOptions {
    pub mode: StaStringView,
    pub limit: usize,
    pub threshold: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct StaMatchOptions {
    pub candidate_limit: usize,
    pub search_threshold: f32,
    pub result_limit: usize,
    pub has_result_limit: bool,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct StaMatchQueryOptions {
    pub search: StaSearchOptions,
    pub result_limit: usize,
    pub has_result_limit: bool,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct StaSearchItem {
    pub id: u64,
    pub score: f32,
}

#[repr(C)]
pub struct StaSearchResult {
    pub items: *mut StaSearchItem,
    pub len: usize,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct StaScoredCandidate {
    pub id: u64,
    pub score: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct StaMatchWinner {
    pub item: StaScoredCandidate,
    pub has_value: bool,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct StaMatchResult {
    pub items: *mut StaScoredCandidate,
    pub items_len: usize,
    pub top: *mut StaScoredCandidate,
    pub top_len: usize,
    pub winner: StaMatchWinner,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct StaShindenMatchResult {
    pub shinden_id: u64,
    pub result: StaMatchResult,
}

#[repr(C)]
pub struct StaMatchListResult {
    pub entries: *mut StaShindenMatchResult,
    pub len: usize,
    pub total: usize,
    pub winners: usize,
    pub has_top: usize,
    pub unmatched: usize,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct StaMatchSelection {
    pub shinden_id: u64,
    pub database_id: u64,
}

#[repr(C)]
pub struct StaExportResult {
    pub path: *mut c_char,
    pub exported_count: usize,
}

pub fn cstring_lossy(message: &str) -> CString {
    let bytes = message
        .as_bytes()
        .iter()
        .copied()
        .filter(|&byte| byte != 0)
        .collect::<Vec<_>>();

    CString::new(bytes).unwrap()
}

pub fn ok() -> StaError {
    StaError {
        status: StaStatus::StaStatusOk,
        message: ptr::null_mut(),
    }
}

pub fn error_result(status: StaStatus, message: &str) -> StaError {
    StaError {
        status,
        message: cstring_lossy(message).into_raw(),
    }
}

pub fn into_raw_string(value: impl AsRef<str>) -> *mut c_char { cstring_lossy(value.as_ref()).into_raw() }

pub fn string_view(value: &str) -> StaStringView {
    StaStringView {
        ptr: value.as_ptr().cast(),
        len: value.len(),
    }
}

pub fn optional_string_view(value: Option<&str>) -> StaStringView {
    value.map_or(
        StaStringView {
            ptr: ptr::null(),
            len: 0,
        },
        string_view,
    )
}

pub fn optional_i32(value: Option<i32>) -> StaOptionalI32 {
    StaOptionalI32 {
        value: value.unwrap_or_default(),
        has_value: value.is_some(),
    }
}

pub fn optional_f32(value: Option<f32>) -> StaOptionalF32 {
    StaOptionalF32 {
        value: value.unwrap_or_default(),
        has_value: value.is_some(),
    }
}

pub fn optional_date(value: Option<NaiveDate>) -> StaOptionalDate {
    match value {
        Some(value) => StaOptionalDate {
            year: value.year(),
            month: value.month(),
            day: value.day(),
            has_value: true,
        },
        None => StaOptionalDate {
            year: 0,
            month: 0,
            day: 0,
            has_value: false,
        },
    }
}

pub fn string_view_array<'a>(values: impl IntoIterator<Item = &'a str>) -> StaStringViewArray {
    let mut entries = values.into_iter().map(string_view).collect::<Vec<_>>();
    entries.shrink_to_fit();
    let len = entries.len();
    let entries = entries.leak().as_mut_ptr();
    StaStringViewArray { entries, len }
}

pub fn empty_match_result() -> StaMatchResult {
    StaMatchResult {
        items: ptr::null_mut(),
        items_len: 0,
        top: ptr::null_mut(),
        top_len: 0,
        winner: StaMatchWinner {
            item: StaScoredCandidate { id: 0, score: 0.0 },
            has_value: false,
        },
    }
}

pub fn with_driver_out_result<T>(
    driver: *mut StaDriver,
    out: *mut T,
    f: impl FnOnce(&StaDriver) -> Result<T, String>,
) -> StaError {
    if driver.is_null() {
        return error_result(StaStatus::StaStatusNullPointer, "driver pointer is null");
    }
    if out.is_null() {
        return error_result(StaStatus::StaStatusNullPointer, "output pointer is null");
    }

    let Some(driver) = driver::as_ref(driver) else {
        return error_result(StaStatus::StaStatusNullPointer, "driver pointer is null");
    };

    match catch_unwind(AssertUnwindSafe(|| f(driver))) {
        Ok(Ok(value)) if !driver.is_aborted() => {
            unsafe {
                *out = value;
            }
            ok()
        },
        Ok(Ok(_)) => error_result(StaStatus::StaStatusError, "driver call aborted"),
        Ok(Err(error)) => error_result(StaStatus::StaStatusError, &error),
        Err(_) => error_result(StaStatus::StaStatusPanic, "driver call panicked"),
    }
}

/// # Safety
/// The pointer must have been allocated by [`CString::into_raw`].
pub unsafe fn free_string(value: *mut c_char) {
    if !value.is_null() {
        drop(unsafe { CString::from_raw(value) });
    }
}

/// # Safety
/// Takes ownership of all owned strings inside `value`.
pub unsafe fn free_database_info(value: StaDatabaseInfo) {
    unsafe {
        free_string(value.last_update);
        free_string(value.release);
        free_string(value.sha256);
        free_string(value.path);
    }
}

/// # Safety
/// Takes ownership of a string view array. The strings themselves are borrowed.
pub unsafe fn free_string_view_array(value: StaStringViewArray) {
    if !value.entries.is_null() {
        drop(unsafe { Vec::from_raw_parts(value.entries, value.len, value.len) });
    }
}

/// # Safety
/// Takes ownership of database entry arrays. String pointers are borrowed from driver-owned data.
pub unsafe fn free_anime_database(value: StaAnimeDatabase) {
    if !value.entries.is_null() {
        let entries = unsafe { Vec::from_raw_parts(value.entries, value.len, value.len) };
        for entry in entries {
            unsafe {
                free_string_view_array(entry.sources);
                free_string_view_array(entry.synonyms);
                free_string_view_array(entry.normalized_synonyms);
                free_string_view_array(entry.studios);
                free_string_view_array(entry.producers);
                free_string_view_array(entry.related_anime);
                free_string_view_array(entry.tags);
            }
        }
    }
}

/// # Safety
/// Takes ownership of the list entry array. String pointers are borrowed from driver-owned list data.
pub unsafe fn free_shinden_list(value: StaShindenList) {
    if !value.entries.is_null() {
        drop(unsafe { Vec::from_raw_parts(value.entries, value.len, value.len) });
    }
}

/// # Safety
/// Takes ownership of id array.
pub unsafe fn free_id_list(value: StaIdList) {
    if !value.entries.is_null() {
        drop(unsafe { Vec::from_raw_parts(value.entries, value.len, value.len) });
    }
}

/// # Safety
/// Takes ownership of search result item array.
pub unsafe fn free_search_result(value: StaSearchResult) {
    if !value.items.is_null() {
        drop(unsafe { Vec::from_raw_parts(value.items, value.len, value.len) });
    }
}

/// # Safety
/// Takes ownership of match result arrays.
pub unsafe fn free_match_result(value: StaMatchResult) {
    if !value.items.is_null() {
        drop(unsafe { Vec::from_raw_parts(value.items, value.items_len, value.items_len) });
    }
    if !value.top.is_null() {
        drop(unsafe { Vec::from_raw_parts(value.top, value.top_len, value.top_len) });
    }
}

/// # Safety
/// Takes ownership of match list result arrays.
pub unsafe fn free_match_list_result(value: StaMatchListResult) {
    if !value.entries.is_null() {
        let entries = unsafe { Vec::from_raw_parts(value.entries, value.len, value.len) };
        for entry in entries {
            unsafe {
                free_match_result(entry.result);
            }
        }
    }
}

/// # Safety
/// Takes ownership of all owned strings inside `value`.
pub unsafe fn free_export_result(value: StaExportResult) {
    unsafe {
        free_string(value.path);
    }
}
