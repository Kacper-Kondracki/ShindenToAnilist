use std::{
    path::Path,
    sync::Arc,
};

use arc_swap::{
    ArcSwap,
    Guard,
};
use shinden_to_anilist_core::{
    database::{
        AnimeDatabase,
        AnimeDatabaseLoad,
        DatabaseError,
    },
    searcher::DefaultSearcher,
};

pub mod pb {
    tonic::include_proto!("shinden_to_anilist.v1");
}

pub mod error;
mod export;
pub mod http;
pub mod mapper;
mod matching;
pub mod server;
mod source;

pub use server::ShindenToAnilist;

#[derive(Debug)]
pub struct Snapshot<T> {
    version: u64,
    value: Option<Arc<T>>,
}

impl<T> Snapshot<T> {
    pub fn version(&self) -> u64 { self.version }
    pub fn is_some(&self) -> bool { self.value.is_some() }
    pub fn is_none(&self) -> bool { self.value.is_none() }
    pub fn get(&self) -> Option<&T> { self.value.as_deref() }
    pub fn get_arc(&self) -> Option<Arc<T>> { self.value.clone() }
}

#[derive(Debug)]
pub struct VersionedArcOption<T> {
    inner: Arc<ArcSwap<Snapshot<T>>>,
}

impl<T> Default for VersionedArcOption<T> {
    fn default() -> Self { Self::empty() }
}

impl<T> Clone for VersionedArcOption<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T> VersionedArcOption<T> {
    pub fn empty() -> Self {
        Self {
            inner: Arc::new(ArcSwap::from_pointee(Snapshot {
                version: 0,
                value: None,
            })),
        }
    }

    pub fn some(value: T) -> Self { Self::some_arc(Arc::new(value)) }
    pub fn some_arc(value: Arc<T>) -> Self {
        Self {
            inner: Arc::new(ArcSwap::from_pointee(Snapshot {
                version: 1,
                value: Some(value),
            })),
        }
    }

    pub fn load(&self) -> VersionedArcOptionGuard<T> {
        VersionedArcOptionGuard {
            guard: self.inner.load(),
        }
    }

    pub fn version(&self) -> u64 { self.inner.load().version }
    pub fn is_some(&self) -> bool { self.inner.load().value.is_some() }
    pub fn is_none(&self) -> bool { self.inner.load().value.is_none() }

    /// Store a heavy value.
    ///
    /// `T` is moved once into an `Arc<T>`.
    /// RCU retries clone only the `Arc`, never `T`.
    pub fn store(&self, value: T) -> u64 { self.store_arc(Arc::new(value)) }

    /// Store an already shared value.
    pub fn store_arc(&self, value: Arc<T>) -> u64 { self.store_option_arc(Some(value)) }

    /// Ergonomic Option setter.
    ///
    /// - `Some(value)` initializes/replaces.
    /// - `None` deinitializes.
    pub fn store_some(&self, value: Option<T>) -> u64 { self.store_option_arc(value.map(Arc::new)) }

    /// Same as `store_some`, but accepts `Option<Arc<T>>`.
    pub fn store_option_arc(&self, value: Option<Arc<T>>) -> u64 {
        let mut published_version = 0;

        self.inner.rcu(|current| {
            let next_version = current.version.wrapping_add(1);
            published_version = next_version;

            Arc::new(Snapshot {
                version: next_version,
                value: value.clone(),
            })
        });

        published_version
    }

    /// Deinitialize.
    ///
    /// Equivalent to storing `None`.
    pub fn clear(&self) -> u64 { self.store_option_arc(None) }

    /// Replace only if the currently published version matches.
    ///
    /// This is useful when callers loaded version N and want to avoid
    /// overwriting version N+1 published by another thread.
    pub fn store_if_version(&self, expected_version: u64, value: T) -> Result<u64, T> {
        self.store_arc_if_version(expected_version, Arc::new(value))
            .map_err(|arc| match Arc::try_unwrap(arc) {
                Ok(value) => value,
                Err(_) => {
                    panic!("value Arc was unexpectedly shared")
                },
            })
    }

    pub fn store_arc_if_version(&self, expected_version: u64, value: Arc<T>) -> Result<u64, Arc<T>> {
        let mut published_version = None;
        let mut did_store = false;

        self.inner.rcu(|current| {
            if current.version != expected_version {
                did_store = false;
                return Arc::clone(current);
            }

            let next_version = current.version + 1;
            published_version = Some(next_version);
            did_store = true;

            Arc::new(Snapshot {
                version: next_version,
                value: Some(Arc::clone(&value)),
            })
        });

        if did_store {
            Ok(published_version.expect("published version missing"))
        } else {
            Err(value)
        }
    }

    /// Clear only if the currently published version matches.
    pub fn clear_if_version(&self, expected_version: u64) -> bool {
        let mut did_clear = false;

        self.inner.rcu(|current| {
            if current.version != expected_version {
                did_clear = false;
                return Arc::clone(current);
            }

            did_clear = true;

            Arc::new(Snapshot {
                version: current.version.wrapping_add(1),
                value: None,
            })
        });

        did_clear
    }

    /// General RCU update.
    ///
    /// The closure receives the currently published version and value.
    ///
    /// Return:
    /// - `Some(arc)` to publish a value
    /// - `None` to publish an empty/deinitialized state
    ///
    /// Important: this closure may run multiple times under contention.
    pub fn update_rcu<F>(&self, mut f: F) -> u64
    where
        F: FnMut(u64, Option<&T>) -> Option<Arc<T>>,
    {
        let mut published_version = 0;

        self.inner.rcu(|current| {
            let next_value = f(current.version, current.value.as_deref());
            let next_version = current.version.wrapping_add(1);

            published_version = next_version;

            Arc::new(Snapshot {
                version: next_version,
                value: next_value,
            })
        });

        published_version
    }

    /// Convenience closure API for borrowing `&T` without exposing the guard.
    pub fn with<R>(&self, f: impl FnOnce(u64, Option<&T>) -> R) -> R {
        let guard = self.load();
        f(guard.version(), guard.get())
    }

    /// Convenience API when caller wants to keep the value after dropping guard.
    ///
    /// This clones only `Arc<T>`, not `T`.
    pub fn load_arc(&self) -> Option<Arc<T>> { self.inner.load().value.clone() }
}

pub struct VersionedArcOptionGuard<T> {
    guard: Guard<Arc<Snapshot<T>>>,
}

impl<T> VersionedArcOptionGuard<T> {
    pub fn version(&self) -> u64 { self.guard.version }
    pub fn is_some(&self) -> bool { self.guard.value.is_some() }
    pub fn is_none(&self) -> bool { self.guard.value.is_none() }

    /// Borrow the heavy value.
    ///
    /// The returned reference is valid as long as this guard is alive.
    pub fn get(&self) -> Option<&T> { self.guard.value.as_deref() }

    /// Clone only the `Arc<T>`, not `T`.
    pub fn get_arc(&self) -> Option<Arc<T>> { self.guard.value.clone() }
    pub fn snapshot(&self) -> &Snapshot<T> { &self.guard }
}

#[derive(Debug)]
pub struct DatabaseState {
    pub database: AnimeDatabase,
    pub searcher: DefaultSearcher,
}

impl DatabaseState {
    fn load(path: impl AsRef<Path>) -> Result<Self, DatabaseError> {
        let database = AnimeDatabase::get_from_mmap(path)?;
        let searcher = DefaultSearcher::new(&database);

        Ok(Self { database, searcher })
    }
}
