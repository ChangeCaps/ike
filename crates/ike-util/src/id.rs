use std::{any::type_name, hash::Hash, marker::PhantomData, sync::Arc};

pub use uuid::Uuid;

pub struct Id<T = ()> {
    inner: Uuid,
    marker: PhantomData<*const T>,
}

unsafe impl<T> Send for Id<T> {}
unsafe impl<T> Sync for Id<T> {}

impl<T> Id<T> {
    pub const fn from_uuid(inner: Uuid) -> Self {
        Self {
            inner,
            marker: PhantomData,
        }
    }

    pub fn new() -> Self {
        Self::from_uuid(Uuid::new_v4())
    }

    pub fn from_ptr(ptr: *const T) -> Self {
        Self {
            inner: Uuid::from_u128(ptr as u128),
            marker: PhantomData,
        }
    }

    pub fn from_arc(arc: &Arc<T>) -> Self {
        Self::from_ptr(Arc::as_ptr(arc))
    }
}

impl<T> std::fmt::Debug for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Id").field("inner", &self.inner).finish()
    }
}

impl<T> std::fmt::Display for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Id<{}>({})", type_name::<T>(), self.inner)
    }
}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            marker: self.marker.clone(),
        }
    }
}

impl<T> Copy for Id<T> {}

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<T> Eq for Id<T> {}

impl<T> PartialOrd for Id<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.inner.partial_cmp(&other.inner)
    }
}

impl<T> Ord for Id<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl<T> Hash for Id<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}
