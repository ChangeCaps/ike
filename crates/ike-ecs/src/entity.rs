use std::{
    collections::BTreeMap,
    ops::Bound,
    sync::atomic::{AtomicU64, Ordering},
};

use crossbeam::queue::SegQueue;

/// Entities act as an "index" into component storage.
///
/// They contain an index and a generation.
/// When and old index is reallocated the generation of all future allocated entities increments.
/// This is so indices can be reused which saves space and resizes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

/// A set of entities without overlapping `generation`s.
/// Where as a normal set of [`Entity`] could contain two entities with the same `index` but
/// differing `generation`s.
#[derive(Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntitySet {
    inner: BTreeMap<u64, u64>,
}

impl EntitySet {
    pub fn new() -> Self {
        Self {
            inner: BTreeMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn contains(&self, entity: &Entity) -> bool {
        self.inner.get(&entity.index()) == Some(&entity.generation())
    }

    pub fn insert(&mut self, entity: Entity) {
        self.inner.insert(entity.index(), entity.generation());
    }

    pub fn remove(&mut self, entity: &Entity) {
        if self.contains(entity) {
            self.inner.remove(&entity.index());
        }
    }

    /// Gets the first [`Entity`] in self.
    pub fn first(&self) -> Option<Entity> {
        self.iter().next()
    }

    /// Gets the first [`Entity`] after `entity`.
    pub fn after(&self, entity: &Entity) -> Option<Entity> {
        self.inner
            .range((Bound::Excluded(entity.index()), Bound::Unbounded))
            .next()
            .map(|(&index, &generation)| Entity::from_raw(index, generation))
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = Entity> + '_ {
        self.inner
            .iter()
            .map(|(&index, &generation)| Entity::from_raw(index, generation))
    }
}
