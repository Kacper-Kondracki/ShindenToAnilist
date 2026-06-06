use std::{
    ffi::{
        CStr,
        CString,
        c_char,
    },
    panic::{
        AssertUnwindSafe,
        catch_unwind,
    },
    path::Path,
    ptr,
    sync::{
        Mutex,
        atomic::{
            AtomicI64,
            Ordering,
        },
    },
};

use shinden_to_anilist_core::{
    BlockingHttpClient,
    Datelike,
    common::AnimeList,
    database::{
        AnimeStatus,
        AnimeType,
        root_metadata_from_path,
        updater::{
            DatabaseUpdateStatus,
            update_latest_jsonl_from_github_blocking,
        },
    },
    exporter::WatchStatus,
    providers::shinden::{
        ShindenList,
        ShindenListLoad,
    },
};

pub struct StaDriver {
    counter: AtomicI64,
    shinden_list: Mutex<Option<ShindenList>>,
}

impl StaDriver {
    fn new() -> Self {
        Self {
            counter: AtomicI64::new(0),
            shinden_list: Mutex::new(None),
        }
    }
}

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

fn cstring_lossy(message: &str) -> CString {
    let bytes = message
        .as_bytes()
        .iter()
        .copied()
        .filter(|&byte| byte != 0)
        .collect::<Vec<_>>();

    CString::new(bytes).unwrap()
}

fn ok() -> StaError {
    StaError {
        status: StaStatus::StaStatusOk,
        message: ptr::null_mut(),
    }
}

fn error_result(status: StaStatus, message: &str) -> StaError {
    StaError {
        status,
        message: cstring_lossy(message).into_raw(),
    }
}

fn into_raw_string(value: impl AsRef<str>) -> *mut c_char { cstring_lossy(value.as_ref()).into_raw() }

fn string_view(value: &str) -> StaStringView {
    StaStringView {
        ptr: value.as_ptr().cast(),
        len: value.len(),
    }
}

fn optional_string_view(value: Option<&str>) -> StaStringView {
    value.map_or(
        StaStringView {
            ptr: ptr::null(),
            len: 0,
        },
        string_view,
    )
}

fn optional_i32(value: Option<i32>) -> StaOptionalI32 {
    StaOptionalI32 {
        value: value.unwrap_or_default(),
        has_value: value.is_some(),
    }
}

fn optional_date(value: Option<shinden_to_anilist_core::NaiveDate>) -> StaOptionalDate {
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

fn anime_status_label(value: AnimeStatus) -> &'static str {
    match value {
        AnimeStatus::Finished => "finished",
        AnimeStatus::Ongoing => "ongoing",
        AnimeStatus::Upcoming => "upcoming",
        AnimeStatus::Unknown => "unknown",
    }
}

fn anime_type_label(value: AnimeType) -> &'static str {
    match value {
        AnimeType::Tv => "tv",
        AnimeType::Movie => "movie",
        AnimeType::Ova => "ova",
        AnimeType::Ona => "ona",
        AnimeType::Special => "special",
        AnimeType::Unknown => "unknown",
    }
}

fn watch_status_label(value: WatchStatus) -> &'static str {
    match value {
        WatchStatus::Dropped => "dropped",
        WatchStatus::Completed => "completed",
        WatchStatus::Watching => "watching",
        WatchStatus::OnHold => "on_hold",
        WatchStatus::PlanToWatch => "plan_to_watch",
    }
}

fn with_driver_out<T>(driver: *mut StaDriver, out: *mut T, f: impl FnOnce(&StaDriver) -> T) -> StaError {
    if driver.is_null() {
        return error_result(StaStatus::StaStatusNullPointer, "driver pointer is null");
    }
    if out.is_null() {
        return error_result(StaStatus::StaStatusNullPointer, "output pointer is null");
    }

    let driver = unsafe { &*driver };

    match catch_unwind(AssertUnwindSafe(|| f(driver))) {
        Ok(value) => {
            unsafe {
                *out = value;
            }
            ok()
        },
        Err(_) => error_result(StaStatus::StaStatusPanic, "driver call panicked"),
    }
}

fn with_driver_out_result<T>(
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

    let driver = unsafe { &*driver };

    match catch_unwind(AssertUnwindSafe(|| f(driver))) {
        Ok(Ok(value)) => {
            unsafe {
                *out = value;
            }
            ok()
        },
        Ok(Err(error)) => error_result(StaStatus::StaStatusError, &error),
        Err(_) => error_result(StaStatus::StaStatusPanic, "driver call panicked"),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn sta_driver_new() -> *mut StaDriver {
    match catch_unwind(|| Box::into_raw(Box::new(StaDriver::new()))) {
        Ok(driver) => driver,
        Err(_) => ptr::null_mut(),
    }
}

/// # Safety
/// Safe if takes ownership and consumes the object.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sta_driver_free(driver: *mut StaDriver) {
    if !driver.is_null() {
        drop(unsafe { Box::from_raw(driver) });
    }
}

/// # Safety
/// Safe if takes ownership and consumes the object.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sta_string_free(value: *mut c_char) {
    if !value.is_null() {
        drop(unsafe { CString::from_raw(value) });
    }
}

/// # Safety
/// Safe if takes ownership and consumes all strings inside `value`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sta_database_info_free(value: StaDatabaseInfo) {
    unsafe {
        sta_string_free(value.last_update);
        sta_string_free(value.release);
        sta_string_free(value.sha256);
        sta_string_free(value.path);
    }
}

/// # Safety
/// Safe if takes ownership and consumes the list entry array. String pointers are borrowed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sta_shinden_list_free(value: StaShindenList) {
    if !value.entries.is_null() {
        drop(unsafe { Vec::from_raw_parts(value.entries, value.len, value.len) });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn sta_driver_counter_value(driver: *mut StaDriver, out: *mut i64) -> StaError {
    with_driver_out(driver, out, |driver| driver.counter.load(Ordering::Relaxed))
}

#[unsafe(no_mangle)]
pub extern "C" fn sta_driver_counter_increment(
    driver: *mut StaDriver,
    amount: u32,
    out: *mut i64,
) -> StaError {
    with_driver_out(driver, out, |driver| {
        let amount = i64::from(amount);
        driver.counter.fetch_add(amount, Ordering::Relaxed) + amount
    })
}

/// # Safety
/// `path` must be valid C string
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sta_driver_ensure_database(
    driver: *mut StaDriver,
    path: *const c_char,
    out: *mut StaDatabaseInfo,
) -> StaError {
    if path.is_null() {
        return error_result(StaStatus::StaStatusNullPointer, "database path pointer is null");
    }

    let path = match unsafe { CStr::from_ptr(path) }.to_str() {
        Ok(path) => path.to_owned(),
        Err(error) => {
            return error_result(
                StaStatus::StaStatusError,
                &format!("database path is not valid UTF-8: {error}"),
            );
        },
    };

    with_driver_out_result(driver, out, move |_| {
        let update_status = update_latest_jsonl_from_github_blocking(
            shinden_to_anilist_core::BlockingHttpClient::new(),
            &path,
        )
        .map_err(|error| error.to_string())?;

        let metadata = root_metadata_from_path(&path).map_err(|error| error.to_string())?;
        let (release, sha256, updated) = match update_status {
            DatabaseUpdateStatus::UpToDate { release, sha256 } => (release, sha256, false),
            DatabaseUpdateStatus::Updated { release, sha256, .. } => (release, sha256, true),
        };

        Ok(StaDatabaseInfo {
            last_update: into_raw_string(metadata.last_update().to_string()),
            release: into_raw_string(release),
            sha256: into_raw_string(sha256),
            path: into_raw_string(Path::new(&path).display().to_string()),
            updated,
        })
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn sta_driver_load_shinden_list(
    driver: *mut StaDriver,
    user_id: u64,
    out: *mut StaShindenList,
) -> StaError {
    with_driver_out_result(driver, out, move |driver| {
        let list = ShindenList::get_from_shinden_blocking(BlockingHttpClient::new(), user_id)
            .map_err(|error| error.to_string())?;

        let mut shinden_list = driver
            .shinden_list
            .lock()
            .map_err(|_| "shinden list lock is poisoned".to_owned())?;
        *shinden_list = Some(list);

        let list = shinden_list
            .as_ref()
            .ok_or_else(|| "loaded shinden list is unavailable".to_owned())?;

        let mut entries = list
            .values()
            .map(|entry| StaShindenEntry {
                id: entry.id(),
                cover_id: optional_i32(entry.cover_id()),
                title: string_view(entry.title()),
                anime_status: string_view(anime_status_label(entry.anime_status())),
                anime_type: string_view(anime_type_label(entry.anime_type())),
                premiere_date: optional_date(entry.premiere_date()),
                finish_date: optional_date(entry.finish_date()),
                episodes: optional_i32(entry.episodes()),
                is_favourite: entry.is_favourite(),
                watch_status: string_view(watch_status_label(entry.watch_status())),
                watched_episodes: entry.watched_episodes(),
                score: optional_i32(entry.score()),
                note: optional_string_view(entry.note().map(|value| value.as_str())),
                description: optional_string_view(entry.description().map(|value| value.as_str())),
            })
            .collect::<Vec<_>>();

        entries.shrink_to_fit();
        let len = entries.len();
        let entries = entries.leak().as_mut_ptr();

        Ok(StaShindenList { entries, len })
    })
}
