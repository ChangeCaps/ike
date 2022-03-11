use std::{collections::BTreeMap, ops::Bound};

use crate::Entity;

pub struct EntitySetIter<'a> {
    inner: std::collections::btree_map::Iter<'a, u64, u64>,
}

impl<'a> Iterator for EntitySetIter<'a> {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((&index, &generation)) = self.inner.next() {
            Some(Entity::from_raw(index, generation))
        } else {
            None
        }
    }
}

pub struct EntitySetIntoIter {
    inner: std::collections::btree_map::IntoIter<u64, u64>,
}

impl<'a> Iterator for EntitySetIntoIter {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((index, generation)) = self.inner.next() {
            Some(Entity::from_raw(index, generation))
        } else {
            None
        }
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

    pub fn len(&self) -> usize {
        self.inner.len()
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

    pub fn iter(&self) -> EntitySetIter<'_> {
        EntitySetIter {
            inner: self.inner.iter(),
        }
    }

    pub fn retain(&mut self, mut f: impl FnMut(Entity) -> bool) {
        self.inner
            .retain(|&index, &mut generation| f(Entity::from_raw(index, generation)))
    }
}

impl IntoIterator for EntitySet {
    type Item = Entity;

    type IntoIter = EntitySetIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        EntitySetIntoIter {
            inner: self.inner.into_iter(),
        }
    }
}
