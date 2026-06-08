pub mod server;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Versioned<T> {
    pub version: u64,
    pub data: T,
}

impl<T> Versioned<T> {
    pub fn new(data: T) -> Self { Self { version: 0, data } }
    pub fn with_version(version: u64, data: T) -> Self { Self { version, data } }
    pub fn new_inc(previous: &Self, data: T) -> Self {
        Versioned {
            version: previous.version.wrapping_add(1),
            data,
        }
    }
}
