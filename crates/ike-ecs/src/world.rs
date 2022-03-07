use std::{
    any::type_name,
    sync::atomic::{AtomicU64, Ordering},
};

use crate::{
    ChangeTick, CommandQueue, Commands, Component, ComponentRead, ComponentWrite, Entities, Entity,
    FromResources, Query, Res, ResMut, Resource, Resources, WorldQuery,
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

    pub fn get<T: Component>(&self, entity: &Entity) -> Option<ComponentRead<'_, T>> {
        self.entities().read_component(entity)
    }

    pub fn get_mut<T: Component>(&self, entity: &Entity) -> Option<ComponentWrite<'_, T>> {
        self.entities().write_component(entity, self.change_tick())
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

    pub fn insert_resource<T: Resource>(&mut self, resource: T) {
        self.resources_mut().insert(resource);
    }

    pub fn init_resource<T: Resource + FromResources>(&mut self) {
        self.resources_mut().init::<T>();
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
}

// query
impl World {
    pub fn query<Q: WorldQuery>(&self) -> Option<Query<'_, Q>> {
        Query::new(self, self.last_change_tick)
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
