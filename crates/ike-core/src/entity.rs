use std::{
    any::TypeId,
    collections::HashMap,
    sync::atomic::{AtomicU64, Ordering},
};

use crossbeam::queue::SegQueue;
use serde::{Deserialize, Serialize};

use crate::{AnyComponent, ComponentStorage, ReadGuard, WriteGuard};

#[repr(C)]
#[derive(
    Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
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

#[derive(Default)]
pub struct Entities {
    storage: HashMap<TypeId, ComponentStorage>,
    entities: Vec<Entity>,
    index: AtomicU64,
    generation: AtomicU64,
    free_indices: SegQueue<u64>,
}

impl Entities {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn spawn(&mut self, entity: Entity) {
        self.entities.push(entity);
    }

    #[inline]
    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }

    #[inline]
    pub fn reserve_entity(&self) -> Entity {
        let idx = if let Some(idx) = self.free_indices.pop() {
            idx
        } else {
            self.index.fetch_add(1, Ordering::SeqCst)
        };

        Entity {
            idx,
            gen: self.generation.load(Ordering::Acquire),
        }
    }

    #[inline]
    pub fn remove_entity(&mut self, entity: &Entity) {
        self.generation.fetch_add(1, Ordering::Release);
        self.free_indices.push(entity.idx());

        for storage in self.storage.values_mut() {
            unsafe { storage.remove_unchecked_raw(entity) };
        }
    }

    #[inline]
    pub fn storage_raw(&self, type_id: &TypeId) -> Option<&ComponentStorage> {
        self.storage.get(type_id)
    }

    #[inline]
    pub fn storage_raw_mut(&mut self, type_id: &TypeId) -> Option<&mut ComponentStorage> {
        self.storage.get_mut(type_id)
    }

    #[inline]
    pub fn storage<T: AnyComponent>(&self) -> Option<&ComponentStorage> {
        self.storage.get(&TypeId::of::<T>())
    }

    #[inline]
    pub fn storage_mut<T: AnyComponent>(&mut self) -> Option<&mut ComponentStorage> {
        self.storage.get_mut(&TypeId::of::<T>())
    }

    #[inline]
    pub fn dump_borrows(&self) {
        for storage in self.storage.values() {
            if storage.borrow_mut() {
                storage.release_mut();

                println!("{} free", storage.ty_name());
            } else if storage.borrow() {
                storage.release();

                println!("{} shared", storage.ty_name());
            } else {
                println!("{} unique", storage.ty_name());
            }
        }
    }

    #[inline]
    pub fn borrow<T: AnyComponent>(&self) -> bool {
        if let Some(storage) = self.storage::<T>() {
            storage.borrow()
        } else {
            true
        }
    }

    #[inline]
    pub fn borrow_mut<T: AnyComponent>(&self) -> bool {
        if let Some(storage) = self.storage::<T>() {
            storage.borrow_mut()
        } else {
            true
        }
    }

    #[inline]
    pub fn release<T: AnyComponent>(&self) {
        if let Some(storage) = self.storage::<T>() {
            storage.release();
        }
    }

    #[inline]
    pub fn release_mut<T: AnyComponent>(&self) {
        if let Some(storage) = self.storage::<T>() {
            storage.release_mut();
        }
    }

    #[inline]
    pub fn has_component<T: AnyComponent>(&self) -> bool {
        self.storage.contains_key(&TypeId::of::<T>())
    }

    #[inline]
    pub fn contains<T: AnyComponent>(&self, entity: &Entity) -> bool {
        if let Some(storage) = self.storage::<T>() {
            storage.contains(entity)
        } else {
            false
        }
    }

    #[inline]
    pub fn insert<T: AnyComponent>(&mut self, entity: Entity, component: T, change_tick: u64) {
        if !self.has_component::<T>() {
            self.storage
                .insert(TypeId::of::<T>(), ComponentStorage::new::<T>());
        }

        let storage = self.storage_mut::<T>().unwrap();
        unsafe { storage.insert_unchecked(entity, component, change_tick) };
    }

    #[inline]
    pub fn remove<T: AnyComponent>(&mut self, entity: &Entity) -> Option<T> {
        unsafe { self.storage_mut::<T>()?.remove_unchecked(entity) }
    }

    #[inline]
    pub fn get_component<T: AnyComponent>(&self, entity: &Entity) -> Option<ReadGuard<T>> {
        unsafe { self.storage::<T>()?.get_borrowed(entity) }
    }

    #[inline]
    pub fn get_component_mut<T: AnyComponent>(&self, entity: &Entity) -> Option<WriteGuard<T>> {
        unsafe { self.storage::<T>()?.get_borrowed_mut(entity) }
    }
}
