mod database;
mod driver;
mod ffi;
mod labels;
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
    StaDatabaseInfo,
    StaError,
    StaShindenList,
    StaStatus,
};

/// # Safety
/// Safe if takes ownership and consumes the object.
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
/// Safe if takes ownership and consumes the object.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sta_string_free(value: *mut c_char) {
    unsafe {
        ffi::free_string(value);
    }
}

/// # Safety
/// Safe if takes ownership and consumes all strings inside `value`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sta_database_info_free(value: StaDatabaseInfo) {
    unsafe {
        ffi::free_database_info(value);
    }
}

/// # Safety
/// Safe if takes ownership and consumes the list entry array. String pointers are borrowed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sta_shinden_list_free(value: StaShindenList) {
    unsafe {
        ffi::free_shinden_list(value);
    }
}

/// # Safety
/// `path` must be valid C string.
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

    ffi::with_driver_out_result(driver, out, move |_| database::ensure_database(&path))
}

#[unsafe(no_mangle)]
pub extern "C" fn sta_driver_load_shinden_list(
    driver: *mut StaDriver,
    user_id: u64,
    out: *mut StaShindenList,
) -> StaError {
    ffi::with_driver_out_result(driver, out, move |driver| shinden::load_list(driver, user_id))
}
