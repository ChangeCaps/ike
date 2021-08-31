use std::{
    hash::Hash,
    marker::PhantomData,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

pub struct Id<T = ()>(pub u64, PhantomData<fn() -> T>);

impl<T> std::fmt::Debug for Id<T> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Id({})", self.0)
    }
}

impl<T> Clone for Id<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0, PhantomData)
    }
}

impl<T> Copy for Id<T> {}

impl<T> PartialEq for Id<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for Id<T> {}

impl<T> PartialOrd for Id<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<T> Ord for Id<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T> Hash for Id<T> {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<T> Default for Id<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Id<T> {
    #[inline]
    pub fn new() -> Self {
        let inner = NEXT_ID.fetch_add(1, Ordering::SeqCst);

        Self(inner, PhantomData)
    }

    #[inline]
    pub fn cast<U>(self) -> Id<U> {
        Id(self.0, PhantomData)
    }
}

impl<T> From<u64> for Id<T> {
    #[inline]
    fn from(id: u64) -> Self {
        Self(id, PhantomData)
    }
}

impl<T> Into<egui::TextureId> for Id<T> {
    #[inline]
    fn into(self) -> egui::TextureId {
        egui::TextureId::User(self.0)
    }
}

pub trait HasId<T = ()> {
    fn id(&self) -> Id<T>;
}
