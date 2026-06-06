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
/// Takes ownership of the list entry array. String pointers are borrowed from driver-owned list data.
pub unsafe fn free_shinden_list(value: StaShindenList) {
    if !value.entries.is_null() {
        drop(unsafe { Vec::from_raw_parts(value.entries, value.len, value.len) });
    }
}
