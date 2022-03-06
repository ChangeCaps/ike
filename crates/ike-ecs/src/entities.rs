use crate::{
    ChangeTick, Component, ComponentRead, ComponentStorages, ComponentWrite, Entity,
    EntityAllocator, EntitySet,
};

#[derive(Default)]
pub struct Entities {
    storage: ComponentStorages,
    allocator: EntityAllocator,
    entities: EntitySet,
}

impl Entities {
    pub fn storage(&self) -> &ComponentStorages {
        &self.storage
    }

    pub fn storage_mut(&mut self) -> &mut ComponentStorages {
        &mut self.storage
    }

    pub fn entities(&self) -> &EntitySet {
        &self.entities
    }

    /// Allocates an [`Entity`] without spawning it.
    pub fn reserve(&self) -> Entity {
        self.allocator.alloc()
    }

    /// Spawn reserved [`Entity`].
    pub fn spawn_reserved_entity(&mut self, entity: Entity) {
        self.entities.insert(entity);
    }

    /// Allocates and spawns [`Entity`].
    pub fn spawn(&mut self) -> Entity {
        let entity = self.reserve();
        self.spawn_reserved_entity(entity);
        entity
    }

    /// Free [`Entity`] and drops all components on it.
    pub fn despawn(&mut self, entity: &Entity) {
        self.storage_mut().despawn(entity);
        self.allocator.free(*entity);
        self.entities.remove(entity);
    }

    pub fn contains_component<T: Component>(&self, entity: &Entity) -> bool {
        self.storage().contains_component::<T>(entity)
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

    pub fn write_component<T: Component>(
        &self,
        entity: &Entity,
        change_tick: ChangeTick,
    ) -> Option<ComponentWrite<'_, T>> {
        self.storage().write_component(entity, change_tick)
    }
}
