use std::{
    mem,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicU64, AtomicUsize, Ordering},
};

// this file is heavily inspired by hecs
// (https://github.com/Ralith/hecs/blob/master/src/borrow.rs)

const UNIQUE_BIT: usize = !(usize::MAX >> 1);
const COUNTER_MASK: usize = usize::MAX >> 1;

#[derive(Debug)]
pub struct AtomicBorrow(AtomicUsize);

impl AtomicBorrow {
    #[inline]
    pub const fn new() -> Self {
        Self(AtomicUsize::new(0))
    }

    #[inline]
    pub fn borrow(&self) -> bool {
        let prev_value = self.0.fetch_add(1, Ordering::Acquire);

        if prev_value & COUNTER_MASK == COUNTER_MASK {
            panic!("immutable borrow counter overflowed");
        }

        if prev_value & UNIQUE_BIT != 0 {
            self.0.fetch_sub(1, Ordering::Release);
            false
        } else {
            true
        }
    }

    #[inline]
    pub fn borrow_mut(&self) -> bool {
        self.0
            .compare_exchange(0, UNIQUE_BIT, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
    }

    #[inline]
    pub fn release(&self) {
        let value = self.0.fetch_sub(1, Ordering::Release);
        debug_assert!(value != 0, "unbalanced release");
        debug_assert!(value & UNIQUE_BIT == 0, "shared release of unique borrow");
    }

    #[inline]
    pub fn release_mut(&self) {
        let value = self.0.fetch_and(!UNIQUE_BIT, Ordering::Release);
        debug_assert_ne!(value & UNIQUE_BIT, 0, "unique release of shared borrow");
    }
}

pub struct ReadGuard<'a, T: ?Sized> {
    pub(crate) value: &'a T,
    pub(crate) borrow: Vec<&'a AtomicBorrow>,
}

impl<'a, T: ?Sized> ReadGuard<'a, T> {
    #[inline]
    pub(crate) fn forget(mut self) -> Vec<&'a AtomicBorrow> {
        mem::replace(&mut self.borrow, Vec::new())
    }
}

impl<'a, T: ?Sized> Deref for ReadGuard<'a, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'a, T: ?Sized> Drop for ReadGuard<'a, T> {
    #[inline]
    fn drop(&mut self) {
        for borrow in &self.borrow {
            borrow.release()
        }
    }
}

pub struct WriteGuard<'a, T: ?Sized> {
    pub(crate) value: &'a mut T,
    pub(crate) borrow: Vec<&'a AtomicBorrow>,
    pub(crate) change_detection: Option<(&'a AtomicU64, u64)>,
}

impl<'a, T: ?Sized> WriteGuard<'a, T> {
    #[inline]
    pub(crate) fn forget(mut self) -> Vec<&'a AtomicBorrow> {
        mem::replace(&mut self.borrow, Vec::new())
    }

    #[inline]
    pub(crate) fn with_change_detection(&mut self, change_count: &'a AtomicU64, change_tick: u64) {
        self.change_detection = Some((change_count, change_tick));
    }
}

impl<'a, T: ?Sized> Deref for WriteGuard<'a, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'a, T: ?Sized> DerefMut for WriteGuard<'a, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        if let Some((change_count, change_tick)) = self.change_detection {
            change_count.store(change_tick, Ordering::Release);
        }

        self.value
    }
}

impl<'a, T: ?Sized> Drop for WriteGuard<'a, T> {
    #[inline]
    fn drop(&mut self) {
        for borrow in &self.borrow {
            borrow.release_mut();
        }
    }
}

pub struct BorrowLock<T: ?Sized> {
    value: *mut T,
    borrow: AtomicBorrow,
}

unsafe impl<T: ?Sized + Send + Sync> Send for BorrowLock<T> {}
unsafe impl<T: ?Sized + Send + Sync> Sync for BorrowLock<T> {}

impl<T> BorrowLock<T> {
    #[inline]
    pub fn new(value: T) -> Self {
        Self {
            value: Box::into_raw(Box::new(value)),
            borrow: AtomicBorrow::new(),
        }
    }

    #[inline]
    pub fn into_inner(self) -> T {
        let inner = *unsafe { Box::from_raw(self.value) };

        mem::forget(self);

        inner
    }
}

impl<T: ?Sized> BorrowLock<T> {
    #[inline]
    pub fn from_box(boxed: Box<T>) -> Self {
        Self {
            value: Box::into_raw(boxed),
            borrow: AtomicBorrow::new(),
        }
    }

    #[inline]
    pub fn read(&self) -> Option<ReadGuard<T>> {
        if self.borrow.borrow() {
            Some(ReadGuard {
                value: unsafe { &*self.value },
                borrow: vec![&self.borrow],
            })
        } else {
            None
        }
    }

    #[inline]
    pub fn write(&self) -> Option<WriteGuard<T>> {
        if self.borrow.borrow_mut() {
            Some(WriteGuard {
                value: unsafe { &mut *self.value },
                borrow: vec![&self.borrow],
                change_detection: None,
            })
        } else {
            None
        }
    }

    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *self.value }
    }

    #[inline]
    pub fn raw(&self) -> *mut T {
        self.value
    }

    #[inline]
    pub fn into_raw(self) -> *mut T {
        let raw = self.value;

        mem::forget(self);

        raw
    }

    #[inline]
    pub fn into_box(self) -> Box<T> {
        let boxed = unsafe { Box::from_raw(self.value) };

        mem::forget(self);

        boxed
    }
}

impl<T: ?Sized> Drop for BorrowLock<T> {
    #[inline]
    fn drop(&mut self) {
        unsafe { Box::from_raw(self.value) };
    }
}
