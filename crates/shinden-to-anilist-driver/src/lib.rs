mod database;
mod driver;
mod ffi;
mod labels;
mod matcher;
mod shinden;

use std::{
    ffi::{
        CStr,
        c_char,
    },
    panic::catch_unwind,
    ptr,
};

pub use driver::StaDriver;
pub use ffi::{
    StaAnimeDatabase,
    StaDatabaseInfo,
    StaError,
    StaExportResult,
    StaIdList,
    StaMatchListResult,
    StaMatchOptions,
    StaMatchQueryOptions,
    StaMatchResult,
    StaMatchSelection,
    StaSearchOptions,
    StaSearchResult,
    StaShindenList,
    StaStatus,
};

/// # Safety
/// `driver` must be null or a pointer returned by [`sta_driver_new`]. After this
/// call, the pointer is consumed and must not be used again.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sta_driver_free(driver: *mut StaDriver) {
    unsafe {
        driver::free(driver);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn sta_driver_new() -> *mut StaDriver {
    match catch_unwind(driver::new) {
        Ok(driver) => driver,
        Err(_) => ptr::null_mut(),
    }
}

/// # Safety
/// `driver` must be null or a live pointer returned by [`sta_driver_new`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sta_driver_abort(driver: *mut StaDriver) { driver::abort(driver); }

/// # Safety
/// `value` must be null or a string allocated by this library. After this call,
/// the pointer is consumed and must not be used again.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sta_string_free(value: *mut c_char) {
    unsafe {
        ffi::free_string(value);
    }
}

/// # Safety
/// `value` must be a database-info result returned by this library. This call
/// consumes all owned strings inside `value`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sta_database_info_free(value: StaDatabaseInfo) {
    unsafe {
        ffi::free_database_info(value);
    }
}

/// # Safety
/// `value` must be an anime-database result returned by this library. This call
/// consumes the entry arrays; string views are borrowed from those entries and
/// become invalid after the free call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sta_anime_database_free(value: StaAnimeDatabase) {
    unsafe {
        ffi::free_anime_database(value);
    }
}

/// # Safety
/// `value` must be a Shinden-list result returned by this library. This call
/// consumes the entry array; string views are borrowed from those entries and
/// become invalid after the free call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sta_shinden_list_free(value: StaShindenList) {
    unsafe {
        ffi::free_shinden_list(value);
    }
}

/// # Safety
/// `value` must be an id-list result returned by this library. This call
/// consumes the id array.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sta_id_list_free(value: StaIdList) {
    unsafe {
        ffi::free_id_list(value);
    }
}

/// # Safety
/// `value` must be a search result returned by this library. This call consumes
/// the result array.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sta_search_result_free(value: StaSearchResult) {
    unsafe {
        ffi::free_search_result(value);
    }
}

/// # Safety
/// `value` must be a match result returned by this library. This call consumes
/// all result arrays.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sta_match_result_free(value: StaMatchResult) {
    unsafe {
        ffi::free_match_result(value);
    }
}

/// # Safety
/// `value` must be a match-list result returned by this library. This call
/// consumes all nested result arrays.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sta_match_list_result_free(value: StaMatchListResult) {
    unsafe {
        ffi::free_match_list_result(value);
    }
}

/// # Safety
/// `value` must be an export result returned by this library. This call consumes
/// all owned strings inside `value`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sta_export_result_free(value: StaExportResult) {
    unsafe {
        ffi::free_export_result(value);
    }
}

/// # Safety
/// `driver` must be a live pointer returned by [`sta_driver_new`]. `path` must
/// be a valid UTF-8 C string. `out` must be non-null and writable; on success it
/// must be released with [`sta_database_info_free`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sta_driver_ensure_database(
    driver: *mut StaDriver,
    path: *const c_char,
    out: *mut StaDatabaseInfo,
) -> StaError {
    if path.is_null() {
        return ffi::error_result(StaStatus::StaStatusNullPointer, "database path pointer is null");
    }

    let path = match unsafe { CStr::from_ptr(path) }.to_str() {
        Ok(path) => path.to_owned(),
        Err(error) => {
            return ffi::error_result(
                StaStatus::StaStatusError,
                &format!("database path is not valid UTF-8: {error}"),
            );
        },
    };

    ffi::with_driver_out_result(driver, out, move |driver| {
        database::ensure_database(driver, &path)
    })
}

/// # Safety
/// `driver` must be a live pointer returned by [`sta_driver_new`]. `out` must
/// be non-null and writable; on success it must be released with
/// [`sta_id_list_free`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sta_driver_load_shinden_list(
    driver: *mut StaDriver,
    user_id: u64,
    out: *mut StaIdList,
) -> StaError {
    ffi::with_driver_out_result(driver, out, move |driver| shinden::load_list(driver, user_id))
}

/// # Safety
/// `driver` must be a live pointer returned by [`sta_driver_new`]. `view` must
/// be a valid UTF-8 C string naming a loaded-list view. `out` must be non-null
/// and writable; on success it must be released with [`sta_id_list_free`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sta_driver_get_loaded_shinden_entry_ids(
    driver: *mut StaDriver,
    view: *const c_char,
    out: *mut StaIdList,
) -> StaError {
    let view = match parse_c_string(view, "shinden entry id view") {
        Ok(view) => view,
        Err(error) => return error,
    };

    ffi::with_driver_out_result(driver, out, move |driver| shinden::get_entry_ids(driver, &view))
}

/// # Safety
/// `driver` must be a live pointer returned by [`sta_driver_new`]. `ids` must
/// point to `len` entries or be null when `len` is 0. `out` must be non-null and
/// writable; on success it must be released with [`sta_shinden_list_free`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sta_driver_get_loaded_shinden_entries(
    driver: *mut StaDriver,
    ids: *const u64,
    len: usize,
    out: *mut StaShindenList,
) -> StaError {
    let ids = match unsafe { parse_id_slice(ids, len, "shinden entry ids") } {
        Ok(ids) => ids.to_vec(),
        Err(error) => return error,
    };

    ffi::with_driver_out_result(driver, out, move |driver| shinden::get_entries(driver, &ids))
}

/// # Safety
/// `driver` must be a live pointer returned by [`sta_driver_new`]. `ids` must
/// point to `len` entries or be null when `len` is 0. `out` must be non-null and
/// writable; on success it must be released with [`sta_anime_database_free`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sta_driver_get_anime_database_entries(
    driver: *mut StaDriver,
    ids: *const u64,
    len: usize,
    out: *mut StaAnimeDatabase,
) -> StaError {
    let ids = match unsafe { parse_id_slice(ids, len, "database entry ids") } {
        Ok(ids) => ids.to_vec(),
        Err(error) => return error,
    };

    ffi::with_driver_out_result(driver, out, move |driver| {
        database::get_database_entries(driver, &ids)
    })
}

/// # Safety
/// `driver` must be a live pointer returned by [`sta_driver_new`]. `query` and
/// `options.mode` must be valid UTF-8 C/string views for the duration of this
/// call. `out` must be non-null and writable; on success it must be released
/// with [`sta_search_result_free`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sta_driver_search_anime(
    driver: *mut StaDriver,
    query: *const c_char,
    options: StaSearchOptions,
    out: *mut StaSearchResult,
) -> StaError {
    let query = match parse_c_string(query, "search query") {
        Ok(query) => query,
        Err(error) => return error,
    };

    ffi::with_driver_out_result(driver, out, move |driver| {
        matcher::search_anime(driver, &query, options)
    })
}

/// # Safety
/// `driver` must be a live pointer returned by [`sta_driver_new`]. `query` and
/// `options.search.mode` must be valid UTF-8 C/string views for the duration of
/// this call. `out` must be non-null and writable; on success it must be
/// released with [`sta_match_result_free`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sta_driver_match_query(
    driver: *mut StaDriver,
    query: *const c_char,
    options: StaMatchQueryOptions,
    out: *mut StaMatchResult,
) -> StaError {
    let query = match parse_c_string(query, "match query") {
        Ok(query) => query,
        Err(error) => return error,
    };

    ffi::with_driver_out_result(driver, out, move |driver| {
        matcher::match_query(driver, &query, options)
    })
}

/// # Safety
/// `driver` must be a live pointer returned by [`sta_driver_new`]. `out` must
/// be non-null and writable; on success it must be released with
/// [`sta_match_list_result_free`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sta_driver_match_loaded_shinden_list(
    driver: *mut StaDriver,
    options: StaMatchOptions,
    out: *mut StaMatchListResult,
) -> StaError {
    ffi::with_driver_out_result(driver, out, move |driver| {
        matcher::match_loaded_shinden_list(driver, options)
    })
}

/// # Safety
/// `driver` must be a live pointer returned by [`sta_driver_new`]. `path` must
/// be a valid UTF-8 C string. `selections` must point to `len` entries or be
/// null when `len` is 0. `out` must be non-null and writable; on success it must
/// be released with [`sta_export_result_free`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sta_driver_export_matches(
    driver: *mut StaDriver,
    path: *const c_char,
    selections: *const StaMatchSelection,
    len: usize,
    out: *mut StaExportResult,
) -> StaError {
    let path = match parse_c_string(path, "export path") {
        Ok(path) => path,
        Err(error) => return error,
    };

    ffi::with_driver_out_result(driver, out, move |driver| unsafe {
        matcher::export_matches(driver, &path, selections, len)
    })
}

fn parse_c_string(value: *const c_char, label: &str) -> Result<String, StaError> {
    if value.is_null() {
        return Err(ffi::error_result(
            StaStatus::StaStatusNullPointer,
            &format!("{label} pointer is null"),
        ));
    }

    match unsafe { CStr::from_ptr(value) }.to_str() {
        Ok(value) => Ok(value.to_owned()),
        Err(error) => Err(ffi::error_result(
            StaStatus::StaStatusError,
            &format!("{label} is not valid UTF-8: {error}"),
        )),
    }
}

unsafe fn parse_id_slice<'a>(value: *const u64, len: usize, label: &str) -> Result<&'a [u64], StaError> {
    if len == 0 {
        return Ok(&[]);
    }

    if value.is_null() {
        return Err(ffi::error_result(
            StaStatus::StaStatusNullPointer,
            &format!("{label} pointer is null"),
        ));
    }

    Ok(unsafe { std::slice::from_raw_parts(value, len) })
}
