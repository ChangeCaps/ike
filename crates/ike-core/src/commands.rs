use crate::{AnyComponent, Entity, Resource, SpawnNode, World};

pub struct Commands<'a> {
    world: &'a World,
}

impl<'a> Commands<'a> {
    #[inline]
    pub fn new(world: &'a World) -> Self {
        Self { world }
    }

    #[inline]
    pub fn insert_resource<T: Resource>(&self, resource: T) {
        self.world.queue_insert_resource(resource);
    }

    #[inline]
    pub fn init_resource<T: Resource + Default>(&self) {
        self.world.queue_init_resource::<T>();
    }

    #[inline]
    pub fn insert_component<T: AnyComponent>(&self, entity: Entity, component: T) {
        self.world.queue_insert(entity, component);
    }

    #[inline]
    pub fn remove_resource<T: Resource>(&self) {
        self.world.queue_remove_resource::<T>();
    }

    #[inline]
    pub fn spawn_node(&self, name: impl Into<String>) -> SpawnNode<'a> {
        SpawnNode::new(self.world, name)
    }
}
