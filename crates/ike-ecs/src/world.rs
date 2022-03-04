use std::{
    any::type_name,
    sync::atomic::{AtomicU64, Ordering},
};

use crate::{ChangeTick, Entities, Resource, ResourceRead, ResourceWrite, Resources};

#[derive(Default)]
pub struct World {
    entities: Entities,
    resources: Resources,
    change_tick: AtomicU64,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: Entities::default(),
            resources: Resources::default(),
            change_tick: AtomicU64::new(0),
        }
    }

    pub fn entities(&self) -> &Entities {
        &self.entities
    }

    pub fn entities_mut(&mut self) -> &mut Entities {
        &mut self.entities
    }

    pub fn resources(&self) -> &Resources {
        &self.resources
    }

    pub fn resources_mut(&mut self) -> &mut Resources {
        &mut self.resources
    }
}

// resources
impl World {
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
}
