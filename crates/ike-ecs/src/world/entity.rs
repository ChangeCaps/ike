use std::sync::atomic::{AtomicU64, Ordering};

use crossbeam::queue::SegQueue;
use ike_reflect::Reflect;

/// Entities act as an "index" into component storage.
///
/// They contain an index and a generation.
/// When and old index is reallocated the generation of all future allocated entities increments.
/// This is so indices can be reused which saves space and resizes.
#[repr(C)]
#[derive(Clone, Copy, Debug, Reflect, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Entity {
    index: u64,
    generation: u64,
}

impl Entity {
    pub const fn from_raw(index: u64, generation: u64) -> Self {
        Self { index, generation }
    }

    pub const fn index(&self) -> u64 {
        self.index
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }
}

impl std::fmt::Display for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[idx: {}, gen: {}]", self.index, self.generation)
    }
}

#[derive(Default)]
pub struct EntityAllocator {
    index: AtomicU64,
    generation: AtomicU64,
    free_queue: SegQueue<Entity>,
}

impl EntityAllocator {
    pub const fn new() -> Self {
        Self {
            index: AtomicU64::new(0),
            generation: AtomicU64::new(0),
            free_queue: SegQueue::new(),
        }
    }

    pub fn index(&self) -> u64 {
        self.index.load(Ordering::Acquire)
    }

    pub fn generation(&self) -> u64 {
        self.generation.load(Ordering::Acquire)
    }

    pub fn alloc(&self) -> Entity {
        if let Some(entity) = self.free_queue.pop() {
            self.generation
                .fetch_max(entity.generation() + 1, Ordering::AcqRel);

            Entity::from_raw(entity.index, self.generation())
        } else {
            let index = self.index.fetch_add(1, Ordering::AcqRel);
            Entity::from_raw(index, self.generation())
        }
    }

    pub fn free(&self, entity: Entity) {
        self.free_queue.push(entity);
    }
}
