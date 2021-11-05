use std::{any::TypeId, borrow::Cow, collections::HashMap, thread};

use crate::{AnyComponent, BorrowLock, Entity, ReadGuard, World, WriteGuard};

pub(crate) struct OwnedComponent {
    insert: fn(Entity, Box<dyn AnyComponent>, &mut World),
    component: Box<dyn AnyComponent>,
}

impl OwnedComponent {
    pub fn new<T: AnyComponent>(component: T) -> Self {
        fn insert<T: AnyComponent>(
            entity: Entity,
            component: Box<dyn AnyComponent>,
            world: &mut World,
        ) {
            let component: T = *unsafe { Box::from_raw(Box::into_raw(component) as *mut _) };

            world.insert::<T>(entity, component);
        }

        Self {
            insert: insert::<T>,
            component: Box::new(component),
        }
    }

    pub fn insert(self, entity: Entity, world: &mut World) {
        (self.insert)(entity, self.component, world)
    }
}

pub struct Node<'a> {
    pub(crate) name: Cow<'a, String>,
    pub(crate) entity: Entity,
    pub(crate) owned: HashMap<TypeId, BorrowLock<OwnedComponent>>,
    pub(crate) world: &'a World,
}

impl<'a> Node<'a> {
    #[inline]
    pub fn name(&self) -> &String {
        &self.name
    }

    #[inline]
    pub fn entity(&self) -> Entity {
        self.entity
    }

    #[inline]
    pub fn insert<T: AnyComponent>(&mut self, component: T) {
        self.owned.insert(
            TypeId::of::<T>(),
            BorrowLock::new(OwnedComponent::new(component)),
        );
    }

    #[inline]
    pub fn world(&self) -> &'a World {
        self.world
    }

    #[inline]
    pub fn get_component<T: AnyComponent>(&self) -> Option<ReadGuard<T>> {
        let type_id = TypeId::of::<T>();

        if let Some(component) = self.owned.get(&type_id) {
            if let Some(read) = component.read() {
                return Some(ReadGuard {
                    value: unsafe { &*(read.value as *const _ as *const _) },
                    borrow: read.forget(),
                });
            } else {
                return None;
            }
        }

        let storage = self.world.components.get(&type_id)?;

        unsafe { storage.get_borrowed(&self.entity) }
    }

    #[inline]
    pub fn get_component_mut<T: AnyComponent>(&self) -> Option<WriteGuard<T>> {
        let type_id = TypeId::of::<T>();

        if let Some(component) = self.owned.get(&type_id) {
            if let Some(write) = component.write() {
                return Some(WriteGuard {
                    value: unsafe { &mut *(write.value as *mut _ as *mut _) },
                    borrow: write.forget(),
                });
            } else {
                return None;
            }
        }

        let storage = self.world.components.get(&type_id)?;

        unsafe { storage.get_borrowed_mut(&self.entity, self.world.change_tick()) }
    }
}

impl<'a> Drop for Node<'a> {
    #[inline]
    fn drop(&mut self) {
        if !thread::panicking() {
            for (_, owned) in self.owned.drain() {
                self.world.queue_insert_raw(self.entity, owned.into_inner());
            }
        }
    }
}
