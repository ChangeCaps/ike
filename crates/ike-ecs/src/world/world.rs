use std::{
    any::type_name,
    sync::atomic::{AtomicU64, Ordering},
};

use crate::{
    ChangeTick, CommandQueue, Commands, Comp, Component, CompMut, Entities, Entity, Mut,
    Query, QueryFilter, Res, ResMut, Resource, Resources, WorldQuery,
};

#[derive(Default)]
pub struct World {
    entities: Entities,
    resources: Resources,
    change_tick: AtomicU64,
    last_change_tick: ChangeTick,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: Entities::default(),
            resources: Resources::default(),
            change_tick: AtomicU64::new(1),
            last_change_tick: 0,
        }
    }

    pub fn commands<O>(&mut self, f: impl FnOnce(Commands) -> O) -> O {
        let command_queue = CommandQueue::new();
        let commands = Commands::new(self, &command_queue);

        let out = f(commands);

        command_queue.apply(self);

        out
    }
}

// entities
impl World {
    pub fn entities(&self) -> &Entities {
        &self.entities
    }

    pub fn entities_mut(&mut self) -> &mut Entities {
        &mut self.entities
    }

    pub fn component<T: Component>(&self, entity: &Entity) -> Option<Comp<T>> {
        self.entities().read(entity)
    }

    pub fn component_mut<T: Component>(&self, entity: &Entity) -> Option<CompMut<T>> {
        self.entities().write(entity, self.change_tick())
    }

    pub fn get_component_mut<T: Component>(&mut self, entity: &Entity) -> Option<Mut<T>> {
        let change_tick = self.change_tick();
        self.entities_mut().get_mut(entity, change_tick)
    }

    pub fn despawn(&mut self, entity: &Entity) {
        self.entities_mut().despawn(entity);
    }
}

// resources
impl World {
    pub fn resources(&self) -> &Resources {
        &self.resources
    }

    pub fn resources_mut(&mut self) -> &mut Resources {
        &mut self.resources
    }

    pub fn contains_resource<T: Resource>(&self) -> bool {
        self.resources().contains::<T>()
    }

    pub fn insert_resource<T: Resource>(&mut self, resource: T) {
        self.resources_mut().insert(resource);
    }

    pub fn init_resource<T: Resource + FromWorld>(&mut self) {
        if !self.contains_resource::<T>() {
            let resource = T::from_world(self);
            self.insert_resource(resource);
        }
    }

    pub fn remove_resource<T: Resource>(&mut self) -> Option<T> {
        self.resources_mut().remove()
    }

    #[track_caller]
    pub fn resource<T: Resource>(&self) -> Res<T> {
        self.resources().read().expect(&format!(
            "resource '{}' not present in world",
            type_name::<T>()
        ))
    }

    #[track_caller]
    pub fn resource_mut<T: Resource>(&self) -> ResMut<T> {
        self.resources().write().expect(&format!(
            "resource '{}' not present in world",
            type_name::<T>()
        ))
    }

    #[track_caller]
    pub fn resource_or_init<T: Resource + FromWorld>(&mut self) -> Res<T> {
        self.init_resource::<T>();

        self.resource()
    }

    #[track_caller]
    pub fn resource_mut_or_init<T: Resource + FromWorld>(&mut self) -> ResMut<T> {
        self.init_resource::<T>();

        self.resource_mut()
    }
}

// change tick
impl World {
    pub fn increment_change_tick(&self) -> ChangeTick {
        self.change_tick.fetch_add(1, Ordering::AcqRel)
    }

    pub fn change_tick(&self) -> ChangeTick {
        self.change_tick.load(Ordering::Acquire)
    }

    pub fn last_change_tick(&self) -> ChangeTick {
        self.last_change_tick
    }

    pub fn update_last_change_tick(&mut self) {
        self.last_change_tick = self.change_tick();
    }

    pub fn set_last_change_tick(&mut self, last_change_tick: ChangeTick) {
        self.last_change_tick = last_change_tick;
    }
}

// query
impl World {
    pub fn query<Q: WorldQuery>(&self) -> Option<Query<'_, Q>> {
        Query::new(self, self.last_change_tick)
    }

    pub fn query_filter<Q: WorldQuery, F: QueryFilter>(&self) -> Option<Query<Q, F>> {
        Query::new(self, self.last_change_tick)
    }
}

pub trait FromWorld {
    fn from_world(world: &mut World) -> Self;
}

impl<T: Default> FromWorld for T {
    fn from_world(_: &mut World) -> Self {
        T::default()
    }
}

#[cfg(test)]
mod tests {
    use crate::{Component, SparseStorage};

    struct Foo {}

    impl Component for Foo {
        type Storage = SparseStorage;
    }

    #[test]
    fn world_query() {}
}
