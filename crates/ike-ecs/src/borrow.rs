use std::{
    mem,
    ops::{Deref, DerefMut},
    ptr,
    sync::atomic::{AtomicU64, Ordering},
};

use crate::ChangeTick;

const COUNTER_MASK: u64 = u64::MAX >> 1;
const UNIQUE_MASK: u64 = !COUNTER_MASK;

#[derive(Debug, Default)]
pub struct AtomicBorrow {
    borrow: AtomicU64,
}

impl AtomicBorrow {
    pub const fn new() -> Self {
        Self {
            borrow: AtomicU64::new(0),
        }
    }

    pub fn borrow(&self) -> bool {
        let prev_value = self.borrow.fetch_add(1, Ordering::Acquire);

        if prev_value & COUNTER_MASK == COUNTER_MASK {
            panic!("AtomicBorrow borrow counter overflowed");
        }

        if prev_value & UNIQUE_MASK != 0 {
            self.borrow.fetch_sub(1, Ordering::Release);
            false
        } else {
            true
        }
    }

    pub fn borrow_mut(&self) -> bool {
        self.borrow
            .compare_exchange(0, UNIQUE_MASK, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
    }

    pub fn release(&self) {
        let value = self.borrow.fetch_sub(1, Ordering::Release);
        debug_assert!(value != 0, "unbalanced release");
        debug_assert!(value & UNIQUE_MASK == 0, "shared release of unique borrow");
    }

    pub fn release_mut(&self) {
        let value = self.borrow.fetch_and(!UNIQUE_MASK, Ordering::Release);
        debug_assert_ne!(value & UNIQUE_MASK, 0, "unique release of shared borrow")
    }
}

pub struct Comp<'a, T: ?Sized> {
    item: &'a T,
    component: &'a AtomicBorrow,
    storage: &'a AtomicBorrow,
}

impl<'a, T: ?Sized> Comp<'a, T> {
    pub fn new(
        item: &'a T,
        component: &'a AtomicBorrow,
        storage: &'a AtomicBorrow,
    ) -> Option<Self> {
        if !component.borrow() {
            return None;
        }

        if !storage.borrow() {
            component.release();
            return None;
        }

        Some(Self {
            item,
            component,
            storage,
        })
    }

    /// This acts as a fix since implementing coercion is still unstable.
    pub fn map_inner<U: ?Sized>(self, f: impl FnOnce(&'a T) -> &'a U) -> Comp<'a, U> {
        let comp = Comp {
            item: f(self.item),
            component: self.component,
            storage: self.storage,
        };

        mem::forget(self);

        comp
    }
}

impl<'a, T: ?Sized> Deref for Comp<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.item
    }
}

impl<'a, T: ?Sized> Drop for Comp<'a, T> {
    fn drop(&mut self) {
        self.component.release();
        self.storage.release();
    }
}

pub struct CompMut<'a, T: ?Sized> {
    item: &'a mut T,
    component: &'a AtomicBorrow,
    storage: &'a AtomicBorrow,
    component_change_tick: &'a AtomicU64,
    change_tick: ChangeTick,
}

impl<'a, T: ?Sized> CompMut<'a, T> {
    pub fn new(
        item: &'a mut T,
        component: &'a AtomicBorrow,
        storage: &'a AtomicBorrow,
        component_change_tick: &'a AtomicU64,
        change_tick: ChangeTick,
    ) -> Option<Self> {
        if !component.borrow_mut() {
            return None;
        }

        if !storage.borrow() {
            component.release_mut();
            return None;
        }

        Some(Self {
            item,
            component,
            storage,
            component_change_tick,
            change_tick,
        })
    }

    /// This acts as a fix since implementing coercion is still unstable.
    ///
    /// # Note
    /// - This _does not_ mark component as changed.
    pub fn map_inner<U: ?Sized>(self, f: impl FnOnce(&'a mut T) -> &'a mut U) -> CompMut<'a, U> {
        // SAFETY:
        // self.item will never be dereferenced after this
        let item = unsafe { ptr::read(&self.item as *const _) };

        let comp = CompMut {
            item: f(item),
            component: self.component,
            storage: self.storage,
            component_change_tick: self.component_change_tick,
            change_tick: self.change_tick,
        };

        mem::forget(self);

        comp
    }
}

impl<'a, T: ?Sized> Deref for CompMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.item
    }
}

impl<'a, T: ?Sized> DerefMut for CompMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.component_change_tick
            .store(self.change_tick, Ordering::Release);

        self.item
    }
}

impl<'a, T: ?Sized> Drop for CompMut<'a, T> {
    fn drop(&mut self) {
        self.component.release_mut();
        self.storage.release();
    }
}

pub struct Mut<'a, T: ?Sized> {
    item: &'a mut T,
    component_change_tick: &'a AtomicU64,
    change_tick: ChangeTick,
}

impl<'a, T: ?Sized> Mut<'a, T> {
    pub fn new(
        item: &'a mut T,
        component_change_tick: &'a AtomicU64,
        change_tick: ChangeTick,
    ) -> Self {
        Self {
            item,
            component_change_tick,
            change_tick,
        }
    }

    /// Gets inner item without marking the it as changed.
    pub fn as_mut_unchanged(&mut self) -> &mut T {
        self.item
    }
}

impl<'a, T: ?Sized> Deref for Mut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.item
    }
}

impl<'a, T: ?Sized> DerefMut for Mut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.component_change_tick
            .store(self.change_tick, Ordering::Release);

        self.item
    }
}

pub struct Res<'a, T: ?Sized> {
    item: &'a T,
    borrow: &'a AtomicBorrow,
}

impl<'a, T: ?Sized> Res<'a, T> {
    pub fn new(item: &'a T, borrow: &'a AtomicBorrow) -> Option<Self> {
        if borrow.borrow() {
            Some(Self { item, borrow })
        } else {
            None
        }
    }
}

impl<'a, T: ?Sized> Deref for Res<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.item
    }
}

impl<'a, T: ?Sized> Drop for Res<'a, T> {
    fn drop(&mut self) {
        self.borrow.release();
    }
}

pub struct ResMut<'a, T: ?Sized> {
    item: &'a mut T,
    borrow: &'a AtomicBorrow,
}

impl<'a, T: ?Sized> Deref for ResMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.item
    }
}

impl<'a, T: ?Sized> DerefMut for ResMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.item
    }
}

impl<'a, T: ?Sized> ResMut<'a, T> {
    pub fn new(item: &'a mut T, borrow: &'a AtomicBorrow) -> Option<Self> {
        if borrow.borrow_mut() {
            Some(Self { item, borrow })
        } else {
            None
        }
    }
}

impl<'a, T: ?Sized> Drop for ResMut<'a, T> {
    fn drop(&mut self) {
        self.borrow.release_mut();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atomic_borrow() {
        let borrow = AtomicBorrow::new();

        assert!(borrow.borrow());
        assert!(borrow.borrow());

        assert!(!borrow.borrow_mut());

        borrow.release();
        borrow.release();

        assert!(borrow.borrow_mut());

        borrow.release_mut();
    }
}
