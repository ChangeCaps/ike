use std::{
    any::type_name,
    hash::{Hash, Hasher},
    marker::PhantomData,
    path::Path,
    sync::atomic::{AtomicU32, Ordering},
};

use ahash::AHasher;
use ike_ecs::Component;

use crate::Asset;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PathId(u64);

impl<T: AsRef<Path>> From<T> for PathId {
    fn from(path: T) -> Self {
        let mut hasher = AHasher::new_with_keys(42, 69);
        path.as_ref().hash(&mut hasher);
        Self(hasher.finish())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HandleId {
    Path(PathId),
    Id(u64),
}

impl HandleId {
    pub fn random() -> Self {
        Self::Id(rand::random())
    }
}

impl From<&str> for HandleId {
    fn from(path_id: &str) -> Self {
        Self::Path(path_id.into())
    }
}

impl From<&Path> for HandleId {
    fn from(path_id: &Path) -> Self {
        Self::Path(path_id.into())
    }
}

impl From<u64> for HandleId {
    fn from(id: u64) -> Self {
        Self::Id(id)
    }
}

impl From<HandleUntyped> for HandleId {
    fn from(handle: HandleUntyped) -> Self {
        handle.id
    }
}

impl<T: Asset> From<Handle<T>> for HandleId {
    fn from(handle: Handle<T>) -> Self {
        handle.inner.id
    }
}

impl From<&HandleUntyped> for HandleId {
    fn from(handle: &HandleUntyped) -> Self {
        handle.id
    }
}

impl<T: Asset> From<&Handle<T>> for HandleId {
    fn from(handle: &Handle<T>) -> Self {
        handle.inner.id
    }
}

pub struct HandleTracker {
    counter: *mut AtomicU32,
}

unsafe impl Send for HandleTracker {}
unsafe impl Sync for HandleTracker {}

impl HandleTracker {
    pub fn new() -> Self {
        Self {
            counter: Box::into_raw(Box::new(AtomicU32::new(0))),
        }
    }

    pub fn reference_count(&self) -> u32 {
        // SAFETY:
        // self.counter was created by Box and is only deallocated when all trackers are dropped
        unsafe { &*self.counter }.load(Ordering::Acquire)
    }
}

impl Clone for HandleTracker {
    fn clone(&self) -> Self {
        // SAFETY:
        // self.counter was created by Box and is only deallocated when all trackers are dropped
        unsafe { &*self.counter }.fetch_add(1, Ordering::Release);

        Self {
            counter: self.counter,
        }
    }
}

impl Drop for HandleTracker {
    fn drop(&mut self) {
        // SAFETY:
        // self.counter was created by Box and is only deallocated when all trackers are dropped
        let count = unsafe { &*self.counter }.fetch_sub(1, Ordering::AcqRel);

        if count == 0 {
            // SAFETY:
            // self.counter was created by Box and is only deallocated when all trackers are dropped
            unsafe { Box::from_raw(self.counter) };
        }
    }
}

#[derive(Clone)]
pub struct HandleUntyped {
    id: HandleId,
    tracker: Option<HandleTracker>,
}

impl HandleUntyped {
    pub fn new(id: impl Into<HandleId>) -> Self {
        Self {
            id: id.into(),
            tracker: Some(HandleTracker::new()),
        }
    }

    pub fn new_weak(id: impl Into<HandleId>) -> Self {
        Self {
            id: id.into(),
            tracker: None,
        }
    }

    pub fn from_tracker(id: HandleId, tracker: HandleTracker) -> Self {
        Self {
            id,
            tracker: Some(tracker),
        }
    }

    pub fn as_weak(&self) -> Self {
        Self {
            id: self.id,
            tracker: None,
        }
    }

    pub fn reference_count(&self) -> Option<u32> {
        self.tracker
            .as_ref()
            .map(|tracker| tracker.reference_count())
    }
}

impl std::fmt::Debug for HandleUntyped {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HandleUntyped")
            .field("id", &self.id)
            .finish()
    }
}

impl PartialEq for HandleUntyped {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for HandleUntyped {}

impl PartialOrd for HandleUntyped {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl Ord for HandleUntyped {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl Hash for HandleUntyped {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[derive(Component)]
pub struct Handle<T: Asset> {
    inner: HandleUntyped,
    marker: PhantomData<&'static T>,
}

impl<T: Asset> Handle<T> {
    pub fn new(id: impl Into<HandleId>) -> Self {
        Self {
            inner: HandleUntyped::new(id),
            marker: PhantomData,
        }
    }

    pub fn new_weak(id: impl Into<HandleId>) -> Self {
        Self {
            inner: HandleUntyped::new_weak(id),
            marker: PhantomData,
        }
    }

    pub fn from_tracker(ty: HandleId, tracker: HandleTracker) -> Self {
        Self {
            inner: HandleUntyped::from_tracker(ty, tracker),
            marker: PhantomData,
        }
    }

    pub fn as_weak(&self) -> Self {
        Self {
            inner: self.inner.as_weak(),
            marker: PhantomData,
        }
    }

    pub fn cast<U: Asset>(&self) -> Handle<U> {
        Handle {
            inner: HandleUntyped {
                id: self.inner.id,
                tracker: self.inner.tracker.as_ref().map(|_| HandleTracker::new()),
            },
            marker: PhantomData,
        }
    }

    pub fn reference_count(&self) -> Option<u32> {
        self.inner.reference_count()
    }
}

impl<T: Asset> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            marker: self.marker.clone(),
        }
    }
}

impl<T: Asset> std::fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(&format!("Handle<{}>", type_name::<T>()))
            .field("inner", &self.inner)
            .finish()
    }
}

impl<T: Asset> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<T: Asset> Eq for Handle<T> {}

impl<T: Asset> PartialOrd for Handle<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.inner.partial_cmp(&other.inner)
    }
}

impl<T: Asset> Ord for Handle<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl<T: Asset> Hash for Handle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}
