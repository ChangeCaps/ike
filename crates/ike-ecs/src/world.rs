use std::{
    any::type_name,
    sync::atomic::{AtomicU64, Ordering},
};

use crate::{
    ChangeTick, Entities, Query, Resource, ResourceRead, ResourceWrite, Resources, WorldQuery,
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
            change_tick: AtomicU64::new(0),
            last_change_tick: 0,
        }
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

    #[track_caller]
    pub fn resource<T: Resource>(&self) -> ResourceRead<T> {
        self.resources().read().expect(&format!(
            "resource '{}' not present in world",
            type_name::<T>()
        ))
    }

    #[track_caller]
    pub fn resource_mut<T: Resource>(&self) -> ResourceWrite<T> {
        self.resources().write().expect(&format!(
            "resource '{}' not present in world",
            type_name::<T>()
        ))
    }
}

// change tick
impl World {
    pub fn increment_change_tick(&self) {
        self.change_tick.fetch_add(1, Ordering::Release);
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
    use crate::{Component, ComponentStorageKind};

    use super::*;

    struct Foo {}

    impl Component for Foo {
        const STORAGE: ComponentStorageKind = ComponentStorageKind::Sparse;
    }

    #[test]
    fn world_query() {
        let mut world = World::new();

        let mut query = world.query::<&Foo>().unwrap();

        for foo in query.iter() {}
    }
}
