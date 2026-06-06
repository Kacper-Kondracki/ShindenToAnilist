use std::{
    ffi::{c_char, CStr, CString},
    panic::{catch_unwind, AssertUnwindSafe},
    path::Path,
    ptr,
    sync::atomic::{AtomicI64, Ordering},
};

use shinden_to_anilist_core::database::{
    root_metadata_from_path,
    updater::{update_latest_jsonl_from_github_blocking, DatabaseUpdateStatus},
};

pub struct StaDriver {
    counter: AtomicI64,
}

impl StaDriver {
    fn new() -> Self {
        Self {
            counter: AtomicI64::new(0),
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

fn into_raw_string(value: impl AsRef<str>) -> *mut c_char {
    cstring_lossy(value.as_ref()).into_raw()
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
