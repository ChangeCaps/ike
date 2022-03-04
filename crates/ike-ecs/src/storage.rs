use std::{
    any::TypeId,
    collections::{BTreeSet, HashMap},
    mem, ptr,
};

use crate::{
    AtomicBorrow, ChangeTick, Component, ComponentData, ComponentDescriptor, ComponentRead,
    ComponentStorageKind, ComponentWrite, Entity, SparseStorage,
};

#[derive(Default)]
pub struct ComponentStorages {
    pub descriptors: HashMap<TypeId, ComponentDescriptor>,
    pub borrow: HashMap<TypeId, AtomicBorrow>,
    pub sparse: HashMap<TypeId, SparseStorage>,
}

impl ComponentStorages {
    pub fn init_storage<T: Component>(&mut self) {
        match T::STORAGE {
            ComponentStorageKind::Sparse => {
                if !self.sparse.contains_key(&TypeId::of::<T>()) {
                    let desc = ComponentDescriptor::new::<T>();

                    self.sparse
                        .insert(TypeId::of::<T>(), SparseStorage::new(&desc));

                    self.descriptors.insert(TypeId::of::<T>(), desc);

                    self.borrow.insert(TypeId::of::<T>(), AtomicBorrow::new());
                }
            }
        }
    }

    pub fn contains_component<T: Component>(&self, entity: &Entity) -> bool {
        match T::STORAGE {
            ComponentStorageKind::Sparse => {
                if let Some(sparse) = self.sparse.get(&TypeId::of::<T>()) {
                    sparse.contains(entity)
                } else {
                    false
                }
            }
        }
    }

    pub fn insert_component<T: Component>(
        &mut self,
        entity: Entity,
        mut component: T,
        change_tick: ChangeTick,
    ) {
        match T::STORAGE {
            ComponentStorageKind::Sparse => {
                self.init_storage::<T>();

                let sparse = self.sparse.get_mut(&TypeId::of::<T>()).unwrap();

                if sparse.contains(&entity) {
                    // SAFETY:
                    // since sparse contains entity, the component at entity.index() is valid
                    unsafe { sparse.drop_unchecked(&entity) };
                }

                // SAFETY:
                // TypeId ensures that T is valid for sparse
                unsafe {
                    sparse.insert_unchecked(entity, (&mut component as *mut T).cast(), change_tick);
                }

                mem::forget(component);
            }
        }
    }

    pub fn remove_component<T: Component>(&mut self, entity: &Entity) -> Option<T> {
        match T::STORAGE {
            ComponentStorageKind::Sparse => {
                let sparse = self.sparse.get_mut(&TypeId::of::<T>())?;

                if sparse.contains(entity) {
                    let ptr = unsafe { sparse.remove_unchecked(entity) };

                    unsafe { Some(ptr::read(ptr as *mut T)) }
                } else {
                    None
                }
            }
        }
    }

    pub fn get_component_raw<T: Component>(&self, entity: &Entity) -> Option<*mut T> {
        match T::STORAGE {
            ComponentStorageKind::Sparse => {
                let sparse = self.sparse.get(&TypeId::of::<T>())?;

                if sparse.contains(entity) {
                    unsafe { Some(sparse.get_unchecked(entity.index() as usize) as *mut T) }
                } else {
                    None
                }
            }
        }
    }

    /// # Safety
    /// A valid T must be present at entity.
    pub unsafe fn get_data_unchecked<T: Component>(&self, entity: &Entity) -> &ComponentData {
        match T::STORAGE {
            ComponentStorageKind::Sparse => {
                let sparse = self.sparse.get(&TypeId::of::<T>()).unwrap();
                unsafe { sparse.get_data_unchecked(entity.index() as usize) }
            }
        }
    }

    pub fn read_component<'a, T: Component>(
        &'a self,
        entity: &Entity,
    ) -> Option<ComponentRead<'a, T>> {
        let item = unsafe { &*(self.get_component_raw(entity)?) };
        let data = unsafe { self.get_data_unchecked::<T>(entity) };

        ComponentRead::new(item, &data.borrow, self.get_borrow::<T>()?)
    }

    pub fn write_component<'a, T: Component>(
        &'a self,
        entity: &Entity,
        change_tick: ChangeTick,
    ) -> Option<ComponentWrite<'a, T>> {
        let item = unsafe { &mut *(self.get_component_raw(entity)?) };
        let data = unsafe { self.get_data_unchecked::<T>(entity) };

        ComponentWrite::new(
            item,
            &data.borrow,
            self.get_borrow::<T>()?,
            &data.ticks.changed_raw(),
            change_tick,
        )
    }

    pub fn despawn(&mut self, entity: &Entity) {
        for sparse in self.sparse.values_mut() {
            if sparse.contains(entity) {
                unsafe { sparse.drop_unchecked(entity) };
            }
        }
    }

    pub fn borrow_storage<T: Component>(&self) -> bool {
        if let Some(borrow) = self.borrow.get(&TypeId::of::<T>()) {
            borrow.borrow()
        } else {
            true
        }
    }

    pub fn borrow_storage_mut<T: Component>(&self) -> bool {
        if let Some(borrow) = self.borrow.get(&TypeId::of::<T>()) {
            borrow.borrow_mut()
        } else {
            true
        }
    }

    pub fn release_storage<T: Component>(&self) {
        if let Some(borrow) = self.borrow.get(&TypeId::of::<T>()) {
            borrow.release();
        }
    }

    pub fn release_storage_mut<T: Component>(&self) {
        if let Some(borrow) = self.borrow.get(&TypeId::of::<T>()) {
            borrow.release_mut();
        }
    }

    pub fn get_borrow<T: Component>(&self) -> Option<&AtomicBorrow> {
        self.borrow.get(&TypeId::of::<T>())
    }

    pub fn get_entities<T: Component>(&self) -> Option<&BTreeSet<Entity>> {
        match T::STORAGE {
            ComponentStorageKind::Sparse => Some(self.sparse.get(&TypeId::of::<T>())?.entities()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    #[allow(unused)]
    struct Foo {
        a: i32,
        b: f64,
    }

    impl Component for Foo {
        const STORAGE: ComponentStorageKind = ComponentStorageKind::Sparse;
    }

    #[derive(Default)]
    #[allow(unused)]
    struct Bar {
        a: i64,
        b: Vec<u32>,
    }

    impl Component for Bar {
        const STORAGE: ComponentStorageKind = ComponentStorageKind::Sparse;
    }

    #[test]
    fn component_storages() {
        let mut storage = ComponentStorages::default();

        let e0 = Entity::from_raw(0, 0);
        let e1 = Entity::from_raw(1, 0);

        let change_tick = 0;

        storage.insert_component(e0, Foo { a: 12, b: 42.69 }, change_tick);
        storage.insert_component(e1, Foo::default(), change_tick);
        storage.insert_component(e0, Bar::default(), change_tick);

        assert!(storage.read_component::<Foo>(&e0).is_some());
        assert!(storage.read_component::<Foo>(&e1).is_some());
        assert!(storage.read_component::<Bar>(&e0).is_some());
        assert!(storage.read_component::<Bar>(&e1).is_none());

        assert_eq!(storage.read_component::<Foo>(&e0).unwrap().a, 12);
        assert_eq!(storage.read_component::<Foo>(&e1).unwrap().a, 0);
    }
}
