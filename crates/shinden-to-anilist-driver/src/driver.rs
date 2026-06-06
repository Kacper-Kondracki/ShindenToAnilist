use std::sync::Mutex;

use shinden_to_anilist_core::providers::shinden::ShindenList;

pub struct StaDriver {
    shinden_list: Mutex<Option<ShindenList>>,
}

impl StaDriver {
    pub fn new() -> Self {
        Self {
            shinden_list: Mutex::new(None),
        }
    }

    pub(crate) fn shinden_list(&self) -> &Mutex<Option<ShindenList>> { &self.shinden_list }
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
