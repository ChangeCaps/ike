use std::{
    ops::{Deref, DerefMut},
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

pub struct ComponentRead<'a, T> {
    item: &'a T,
    component: &'a AtomicBorrow,
    storage: &'a AtomicBorrow,
}

impl<'a, T> ComponentRead<'a, T> {
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
}

impl<'a, T> Deref for ComponentRead<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.item
    }
}

impl<'a, T> Drop for ComponentRead<'a, T> {
    fn drop(&mut self) {
        self.component.release();
        self.storage.release();
    }
}

pub struct ComponentWrite<'a, T> {
    item: &'a mut T,
    component: &'a AtomicBorrow,
    storage: &'a AtomicBorrow,
    component_change_tick: &'a AtomicU64,
    change_tick: ChangeTick,
}

impl<'a, T> ComponentWrite<'a, T> {
    pub fn new(
        item: &'a mut T,
        component: &'a AtomicBorrow,
        storage: &'a AtomicBorrow,
        component_change_tick: &'a AtomicU64,
        change_tick: ChangeTick,
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
            component_change_tick,
            change_tick,
        })
    }
}

impl<'a, T> Deref for ComponentWrite<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.item
    }
}

impl<'a, T> DerefMut for ComponentWrite<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.component_change_tick
            .store(self.change_tick, Ordering::Release);

        self.item
    }
}

impl<'a, T> Drop for ComponentWrite<'a, T> {
    fn drop(&mut self) {
        self.component.release_mut();
        self.storage.release_mut();
    }
}

pub struct Mut<'a, T> {
    item: &'a mut T,
    component_change_tick: &'a AtomicU64,
    change_tick: ChangeTick,
}

impl<'a, T> Mut<'a, T> {
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
}

impl<'a, T> Deref for Mut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.item
    }
}

impl<'a, T> DerefMut for Mut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.component_change_tick
            .store(self.change_tick, Ordering::Release);

        self.item
    }
}

pub struct ResourceRead<'a, T> {
    item: &'a T,
    borrow: &'a AtomicBorrow,
}

impl<'a, T> ResourceRead<'a, T> {
    pub fn new(item: &'a T, borrow: &'a AtomicBorrow) -> Option<Self> {
        if borrow.borrow() {
            Some(Self { item, borrow })
        } else {
            None
        }
    }
}

impl<'a, T> Deref for ResourceRead<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.item
    }
}

impl<'a, T> Drop for ResourceRead<'a, T> {
    fn drop(&mut self) {
        self.borrow.release();
    }
}

pub struct ResourceWrite<'a, T> {
    item: &'a mut T,
    borrow: &'a AtomicBorrow,
}

impl<'a, T> Deref for ResourceWrite<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.item
    }
}

impl<'a, T> DerefMut for ResourceWrite<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.item
    }
}

impl<'a, T> ResourceWrite<'a, T> {
    pub fn new(item: &'a mut T, borrow: &'a AtomicBorrow) -> Option<Self> {
        if borrow.borrow_mut() {
            Some(Self { item, borrow })
        } else {
            None
        }
    }
}

impl<'a, T> Drop for ResourceWrite<'a, T> {
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
