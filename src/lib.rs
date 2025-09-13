use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Weak};

use async_trait::async_trait;
use dashmap::DashMap;
#[cfg(feature = "logging")]
use tracing::{debug, info, trace};

pub mod observers;

/// The `Observer` trait defines the contract for any type that wants to be notified of events.
///
/// It uses async methods in traits to allow for observers that perform asynchronous operations.
#[async_trait]
pub trait Observer<T>: Send + Sync {
    /// The update method is called by the `Subject` when a new event occurs.
    ///
    /// It receives a reference to the data and performs its logic.
    async fn update(&self, data: &T);
}

/// A type alias for the internal list of observers, to improve readability.
/// `DashMap` provides a highly concurrent, lock-free way to store key-value pairs.
/// Here, the key is the observer ID, and the value is the observer itself.
type ObserverList<T> = DashMap<u64, Arc<dyn Observer<T>>>;

// A private struct that holds the internal state of the Subject.
// This allows us to use a `Weak` reference to it from the handle.
struct SubjectInner<T> {
    observers: ObserverList<T>,
    next_observer_id: AtomicU64,
}

/// A handle for an `Observer`, used to uniquely identify and detach it from the `Subject`.
///
/// When this handle goes out of scope, its `Drop` implementation will automatically
/// detach the associated observer from the `Subject`.
#[derive(Debug)]
pub struct ObserverHandle<T> {
    id: u64,
    subject_weak: Weak<SubjectInner<T>>,
}

impl<T> ObserverHandle<T> {
    pub fn get_id(&self) -> u64 {
        self.id
    }
}

impl<T> Drop for ObserverHandle<T> {
    fn drop(&mut self) {
        if let Some(subject_arc) = self.subject_weak.upgrade() {
            if subject_arc.observers.remove(&self.id).is_some() {
                #[cfg(feature = "logging")]
                info!(
                    "Observer with ID {} automatically detached by drop.",
                    self.id
                );
            }
        }
    }
}

/// The `Subject` struct manages the list of observers and notifies them of events.
///
/// It is thread-safe and can be cloned to be used across multiple threads or async tasks.
/// The `Subject` should be created and wrapped in an `Arc` for use.
pub struct Subject<T> {
    inner: Arc<SubjectInner<T>>,
}

impl<T: Send + Sync + 'static> Subject<T> {
    /// Creates a new `Subject` with an empty list of observers.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(SubjectInner {
                observers: DashMap::new(),
                next_observer_id: AtomicU64::new(0),
            }),
        }
    }

    /// Attaches an `Observer` to the `Subject`.
    ///
    /// The observer must be wrapped in `Arc` for shared ownership. Returns a unique handle
    /// that will automatically detach the observer when dropped.
    pub fn attach(&self, observer: Arc<dyn Observer<T>>) -> ObserverHandle<T> {
        let id = self.inner.next_observer_id.fetch_add(1, Ordering::Relaxed);
        self.inner.observers.insert(id, observer);
        #[cfg(feature = "logging")]
        info!("Attached new observer with ID {}.", id);

        ObserverHandle {
            id,
            subject_weak: Arc::downgrade(&self.inner),
        }
    }

    /// Explicitly detaches an `Observer` from the `Subject` using its handle.
    ///
    /// This method consumes the handle and returns `true` if the observer was found
    /// and detached, `false` otherwise.
    pub fn detach(&self, handle_id: u64) -> bool {
        if self.inner.observers.remove(&handle_id).is_some() {
            #[cfg(feature = "logging")]
            info!("Observer with ID {} explicitly detached.", handle.id);
            true
        } else {
            #[cfg(feature = "logging")]
            debug!(
                "Could not find observer with ID {} for explicit detachment.",
                handle.id
            );
            false
        }
    }

    /// Notifies all attached observers of an event.
    ///
    /// The `notify` method takes data by reference and runs each observer's `update` method
    /// concurrently using `futures::future::join_all`. This ensures that a slow observer
    /// does not block others.
    pub async fn notify(&self, data: &T) {
        let observer_arcs: Vec<Arc<dyn Observer<T>>> = {
            self.inner
                .observers
                .iter()
                .map(|item| item.clone())
                .collect()
        }; // `DashMap` iterator is safe and does not need a lock

        #[cfg(feature = "logging")]
        trace!("Notifying {} observers...", observer_arcs.len());
        let mut futures = Vec::new();
        for observer in observer_arcs {
            let future = async move {
                observer.update(data).await;
            };
            futures.push(future);
        }
        futures::future::join_all(futures).await;
    }
}

// Implement `Clone` to allow creating multiple `Arc`s to the same Subject.
impl<T> Clone for Subject<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

// Add a Default implementation as suggested by clippy.
impl<T: Send + Sync + 'static> Default for Subject<T> {
    fn default() -> Self {
        Self::new()
    }
}
