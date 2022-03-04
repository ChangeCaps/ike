use std::collections::BTreeSet;

use crate::{
    ChangeTick, Component, ComponentRead, ComponentStorages, ComponentWrite, Entity,
    EntityAllocator,
};

#[derive(Default)]
pub struct Entities {
    storage: ComponentStorages,
    allocator: EntityAllocator,
    entities: BTreeSet<Entity>,
}

impl Entities {
    pub fn storage(&self) -> &ComponentStorages {
        &self.storage
    }

    pub fn storage_mut(&mut self) -> &mut ComponentStorages {
        &mut self.storage
    }

    pub fn reserve(&self) -> Entity {
        self.allocator.alloc()
    }

    pub fn spawn(&mut self) -> Entity {
        let entity = self.reserve();
        self.entities.insert(entity);
        entity
    }

    pub fn despawn(&mut self, entity: &Entity) {
        self.storage_mut().despawn(entity);
        self.allocator.free(*entity);
        self.entities.remove(entity);
    }

    pub fn insert<T: Component>(&mut self, entity: &Entity, component: T, change_tick: ChangeTick) {
        self.storage_mut()
            .insert_component(*entity, component, change_tick);
    }

    pub fn remove<T: Component>(&mut self, entity: &Entity) -> Option<T> {
        self.storage_mut().remove_component(entity)
    }

    pub fn read_component<T: Component>(&self, entity: &Entity) -> Option<ComponentRead<'_, T>> {
        self.storage().read_component(entity)
    }

    pub fn write_component<T: Component>(&self, entity: &Entity) -> Option<ComponentWrite<'_, T>> {
        self.storage().write_component(entity)
    }
}
