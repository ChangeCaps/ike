use std::{
    collections::HashMap,
    sync::atomic::{AtomicU64, Ordering},
};

use crate::{
    CommandQueue, Commands, Entities, Entity, ReadGuard, Resource, Resources, WorldRef, WriteGuard,
};

pub struct World {
    entities: Entities,
    nodes: HashMap<Entity, String>,
    resources: Resources,
    change_tick: AtomicU64,
    last_change_tick: u64,
}

impl Default for World {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    #[inline]
    pub fn new() -> Self {
        Self {
            entities: Entities::new(),
            nodes: HashMap::new(),
            resources: Resources::new(),
            change_tick: AtomicU64::new(1),
            last_change_tick: 0,
        }
    }

    #[inline]
    pub fn entities(&self) -> &Entities {
        &self.entities
    }

    #[inline]
    pub fn entities_mut(&mut self) -> &mut Entities {
        &mut self.entities
    }

    #[inline]
    pub fn insert_resource<T: Resource>(&mut self, resource: T) {
        self.resources.insert(resource);
    }

    #[inline]
    pub fn init_resource<T: Resource + Default>(&mut self) {
        if !self.resources.contains::<T>() {
            self.resources.insert(T::default());
        }
    }

    #[inline]
    pub fn has_resource<T: Resource>(&self) -> bool {
        self.resources.contains::<T>()
    }

    #[inline]
    pub fn read_resource<T: Resource>(&self) -> Option<ReadGuard<T>> {
        self.resources.read()
    }

    #[inline]
    pub fn write_resource<T: Resource>(&self) -> Option<WriteGuard<T>> {
        self.resources.write()
    }

    #[inline]
    pub fn remove_resource<T: Resource>(&mut self) -> Option<T> {
        self.resources.remove()
    }

    #[inline]
    pub fn resources(&self) -> &Resources {
        &self.resources
    }

    #[inline]
    pub fn nodes(&self) -> &HashMap<Entity, String> {
        &self.nodes
    }

    #[inline]
    pub fn get_node_name(&self, entity: &Entity) -> Option<&String> {
        self.nodes.get(entity)
    }

    #[inline]
    pub fn get_node_name_mut(&mut self, entity: &Entity) -> Option<&mut String> {
        self.nodes.get_mut(entity)
    }

    #[inline]
    pub fn set_node_name(&mut self, entity: &Entity, name: impl Into<String>) {
        self.nodes.insert(*entity, name.into());
    }

    #[inline]
    pub fn world_ref<O>(&mut self, f: impl FnOnce(WorldRef) -> O) -> O {
        let mut command_queue = CommandQueue::default();
        let commands = Commands::new(self.entities(), &mut command_queue);
        let world_ref = WorldRef::new(self, commands, self.last_change_tick());

        let o = f(world_ref);

        command_queue.apply(self);

        o
    }

    #[inline]
    pub fn clear_trackers(&mut self) {
        self.last_change_tick = self.increment_change_tick();
    }

    #[inline]
    pub fn change_tick(&self) -> u64 {
        self.change_tick.load(Ordering::Acquire)
    }

    #[inline]
    pub fn last_change_tick(&self) -> u64 {
        self.last_change_tick
    }

    #[inline]
    pub fn set_last_change_tick(&mut self, last_change_tick: u64) {
        self.last_change_tick = last_change_tick;
    }

    #[inline]
    pub fn increment_change_tick(&self) -> u64 {
        self.change_tick.fetch_add(1, Ordering::SeqCst)
    }
}
