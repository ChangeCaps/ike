use std::{hash::Hash, marker::PhantomData, path::PathBuf, sync::Arc};

use ike_derive::Reflect;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HandleUntyped {
    Path(PathBuf),
    Id(u64),
}

pub trait IntoHandleUntyped {
    fn into_handle_untyped(self) -> HandleUntyped;
}

impl IntoHandleUntyped for u64 {
    #[inline]
    fn into_handle_untyped(self) -> HandleUntyped {
        HandleUntyped::Id(self)
    }
}

impl IntoHandleUntyped for &str {
    #[inline]
    fn into_handle_untyped(self) -> HandleUntyped {
        HandleUntyped::Path(self.into())
    }
}

impl IntoHandleUntyped for PathBuf {
    #[inline]
    fn into_handle_untyped(self) -> HandleUntyped {
        HandleUntyped::Path(self)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum Inner {
    Tracked(Arc<HandleUntyped>),
    Untracked(HandleUntyped),
}

impl Inner {
    #[inline]
    fn untyped(&self) -> &HandleUntyped {
        match self {
            Self::Tracked(handle) => handle,
            Self::Untracked(handle) => handle,
        }
    }
}

#[derive(Reflect, Serialize, Deserialize)]
#[serde(bound = "T: 'static")]
#[reflect(value)]
pub struct Handle<T: 'static> {
    inner: Inner,
    marker: PhantomData<&'static T>,
}

impl<T> Handle<T> {
    #[inline]
    pub fn new<U: IntoHandleUntyped>(untyped: U) -> Self {
        Self {
            inner: Inner::Tracked(Arc::new(untyped.into_handle_untyped())),
            marker: PhantomData,
        }
    }

    #[inline]
    pub fn new_untracked<U: IntoHandleUntyped>(untyped: U) -> Self {
        Self {
            inner: Inner::Untracked(untyped.into_handle_untyped()),
            marker: PhantomData,
        }
    }

    #[inline]
    pub const fn untracked_from_u64(id: u64) -> Self {
        Self {
            inner: Inner::Untracked(HandleUntyped::Id(id)),
            marker: PhantomData,
        }
    }

    #[inline]
    pub fn new_rand() -> Self {
        Self::new(rand::random::<u64>())
    }

    #[inline]
    pub fn untyped(&self) -> &HandleUntyped {
        self.inner.untyped()
    }

    #[inline]
    pub fn tracked(&self) -> Option<&Arc<HandleUntyped>> {
        match self.inner {
            Inner::Tracked(ref tracked) => Some(tracked),
            _ => None,
        }
    }

    #[inline]
    pub fn shared(&self) -> bool {
        self.tracked()
            .map(|tracked| Arc::strong_count(tracked) > 1)
            .unwrap_or(true)
    }
}

impl<T> Default for Handle<T> {
    #[inline]
    fn default() -> Self {
        Self::new_rand()
    }
}

impl<T> PartialEq for Handle<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.untyped().eq(other.untyped())
    }
}

impl<T> Eq for Handle<T> {}

impl<T> Hash for Handle<T> {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.untyped().hash(state)
    }
}

impl<T> Clone for Handle<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            marker: PhantomData,
        }
    }
}

impl<T> std::fmt::Debug for Handle<T> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}
