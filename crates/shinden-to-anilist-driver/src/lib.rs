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
    sync::atomic::{
        AtomicI64,
        Ordering,
    },
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
