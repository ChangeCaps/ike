use crate::{AnyComponent, Entity, World};

pub struct SpawnNode<'a> {
    name: String,
    entity: Entity,
    world: &'a World,
}

impl<'a> SpawnNode<'a> {
    #[inline]
    pub fn new(world: &'a World, name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            entity: world.create_entity(),
            world,
        }
    }

    #[inline]
    pub fn entity(&self) -> Entity {
        self.entity
    }

    #[inline]
    pub fn insert<T: AnyComponent>(&self, component: T) {
        self.world.queue_insert(self.entity, component);
    }
}

impl<'a> Drop for SpawnNode<'a> {
    #[inline]
    fn drop(&mut self) {
        self.world.queue_set_node_name(self.entity, &self.name);
    }
}
