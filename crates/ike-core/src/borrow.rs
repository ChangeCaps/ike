use std::{
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicUsize, Ordering},
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
        self.0.fetch_sub(1, Ordering::Release);
    }

    #[inline]
    pub fn release_mut(&self) {
        self.0.fetch_and(!UNIQUE_BIT, Ordering::Release);
    }
}

pub struct ReadGuard<'a, T: ?Sized> {
    pub(crate) value: &'a T,
    pub(crate) borrow: &'a AtomicBorrow,
}

impl<'a, T: ?Sized> ReadGuard<'a, T> {
    #[inline]
    pub fn map<U>(self, f: impl FnOnce(&T) -> &U) -> ReadGuard<'a, U> {
        ReadGuard {
            value: f(self.value),
            borrow: self.borrow,
        }
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
        self.borrow.release()
    }
}

pub struct WriteGuard<'a, T: ?Sized> {
    pub(crate) value: &'a mut T,
    pub(crate) borrow: &'a AtomicBorrow,
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
        self.value
    }
}

impl<'a, T: ?Sized> Drop for WriteGuard<'a, T> {
    #[inline]
    fn drop(&mut self) {
        self.borrow.release_mut()
    }
}

pub struct BorrowLock<T: ?Sized> {
    value: *mut T,
    borrow: AtomicBorrow,
}

impl<T> BorrowLock<T> {
    #[inline]
    pub fn new(value: T) -> Self {
        Self {
            value: Box::into_raw(Box::new(value)),
            borrow: AtomicBorrow::new(),
        }
    }
}

impl<T: ?Sized> BorrowLock<T> {
    #[inline]
    pub fn map<U: ?Sized>(self, f: impl FnOnce(*mut T) -> *mut U) -> BorrowLock<U> {
        BorrowLock {
            value: f(self.value),
            borrow: AtomicBorrow::new(),
        }
    }

    #[inline]
    pub fn read(&self) -> Option<ReadGuard<T>> {
        if self.borrow.borrow() {
            Some(ReadGuard {
                value: unsafe { &*self.value },
                borrow: &self.borrow,
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
                borrow: &self.borrow,
            })
        } else {
            None
        }
    }

    #[inline]
    pub fn raw(&self) -> *mut T {
        self.value
    }
}

impl<T: ?Sized> Drop for BorrowLock<T> {
    #[inline]
    fn drop(&mut self) {
        unsafe { Box::from_raw(self.value) };
    }
}
