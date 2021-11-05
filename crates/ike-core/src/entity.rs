use std::sync::atomic::{AtomicU64, Ordering};

use crossbeam::queue::SegQueue;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Entity {
    idx: u64,
    gen: u64,
}

impl Entity {
    #[inline]
    pub fn from_raw(idx: u64, gen: u64) -> Self {
        Entity { idx, gen }
    }

    #[inline]
    pub fn idx(self) -> u64 {
        self.idx
    }

    #[inline]
    pub fn gen(self) -> u64 {
        self.gen
    }
}

#[derive(Debug, Default)]
pub struct EntityRegistry {
    idx: AtomicU64,
    gen: AtomicU64,
    free_indices: SegQueue<u64>,
}

impl EntityRegistry {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn next(&self) -> Entity {
        let idx = if let Some(idx) = self.free_indices.pop() {
            idx
        } else {
            self.idx.fetch_add(1, Ordering::SeqCst)
        };

        Entity {
            idx,
            gen: self.gen.load(Ordering::Acquire),
        }
    }

    #[inline]
    pub fn free(&self, entity: Entity) {
        self.gen.fetch_add(1, Ordering::Release);
        self.free_indices.push(entity.idx);
    }
}
