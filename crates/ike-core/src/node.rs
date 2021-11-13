use std::{any::TypeId, borrow::Cow, collections::HashMap};

use crate::{AnyComponent, BorrowLock, Entity, ReadGuard, WorldRef, WriteGuard};

pub trait OwnedComponent: AnyComponent {
    fn insert(self: Box<Self>, entity: &Entity, world: &WorldRef);
}

impl<T: AnyComponent> OwnedComponent for T {
    fn insert(self: Box<Self>, entity: &Entity, world: &WorldRef) {
        world.insert_component(entity, *self);
    }
}

pub struct Node<'w, 's> {
    pub(crate) name: Cow<'w, String>,
    pub(crate) components: HashMap<TypeId, BorrowLock<dyn OwnedComponent>>,
    pub(crate) entity: Entity,
    pub(crate) world_ref: &'w WorldRef<'w, 's>,
}

impl<'w, 's> Node<'w, 's> {
    #[inline]
    pub fn new(world_ref: &'w WorldRef<'w, 's>, entity: Entity, name: &'w String) -> Self {
        Self {
            name: Cow::Borrowed(name),
            components: HashMap::new(),
            entity,
            world_ref,
        }
    }

    #[inline]
    pub fn name(&self) -> &String {
        &self.name
    }

    #[inline]
    pub fn entity(&self) -> Entity {
        self.entity
    }

    #[inline]
    pub fn world(&self) -> &'w WorldRef {
        self.world_ref
    }

    #[inline]
    pub fn insert<T: AnyComponent>(&mut self, component: T) {
        self.components
            .insert(TypeId::of::<T>(), BorrowLock::from_box(Box::new(component)));
    }

    #[inline]
    pub fn remove<T: AnyComponent>(&mut self) {
        if self.components.remove(&TypeId::of::<T>()).is_none() {
            self.world_ref.remove_component::<T>(&self.entity);
        }
    }

    #[inline]
    pub fn get_component<T: AnyComponent>(&self) -> Option<ReadGuard<T>> {
        if let Some(component) = self.components.get(&TypeId::of::<T>()) {
            let guard = component.read()?;

            return Some(ReadGuard {
                value: unsafe { &*(guard.value as *const _ as *const _) },
                borrow: guard.forget(),
            });
        }

        self.world_ref.get_component(&self.entity)
    }

    #[inline]
    pub fn get_component_mut<T: AnyComponent>(&self) -> Option<WriteGuard<T>> {
        if let Some(component) = self.components.get(&TypeId::of::<T>()) {
            let mut guard = component.write()?;

            return Some(WriteGuard {
                value: unsafe { &mut *(guard.value as *mut _ as *mut _) },
                change_detection: guard.change_detection.take(),
                borrow: guard.forget(),
            });
        }

        self.world_ref.get_component_mut(&self.entity)
    }
}

impl<'w, 's> Drop for Node<'w, 's> {
    #[inline]
    fn drop(&mut self) {
        for (_id, component) in self.components.drain() {
            component.into_box().insert(&self.entity, self.world_ref);
        }
    }
}
