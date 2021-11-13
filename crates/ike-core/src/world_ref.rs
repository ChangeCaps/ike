use crate::{
    AnyComponent, Commands, Entities, Entity, Node, Query, QueryFilter, ReadGuard, Resource, World,
    WorldQuery, WriteGuard,
};

pub struct WorldRef<'w, 's> {
    world: &'w World,
    commands: Commands<'w, 's>,
    last_change_tick: u64,
}

impl<'w, 's> WorldRef<'w, 's> {
    #[inline]
    pub fn new(world: &'w World, commands: Commands<'w, 's>, last_change_tick: u64) -> Self {
        Self {
            world,
            commands,
            last_change_tick,
        }
    }

    #[inline]
    pub fn world(&self) -> &World {
        self.world
    }

    #[inline]
    pub fn commands(&self) -> &Commands {
        &self.commands
    }

    #[inline]
    pub fn entities(&self) -> &Entities {
        self.world.entities()
    }

    #[inline]
    pub fn despawn(&self, entity: &Entity) {
        self.commands.despawn(entity);
    }

    #[inline]
    pub fn get_node(&self, entity: &Entity) -> Option<Node<'_, '_>> {
        let name = self.world.get_node_name(entity)?;

        Some(Node::new(self, *entity, name))
    }

    #[inline]
    pub fn query<Q: WorldQuery>(&self) -> Option<Query<Q>> {
        Query::new(self.world, self.last_change_tick)
    }

    #[inline]
    pub fn query_filter<Q: WorldQuery, F: QueryFilter>(&self) -> Option<Query<Q, F>> {
        Query::new(self.world, self.last_change_tick)
    }

    #[inline]
    pub fn get_component<T: AnyComponent>(&self, entity: &Entity) -> Option<ReadGuard<T>> {
        self.world.entities().get_component(entity)
    }

    #[inline]
    pub fn get_component_mut<T: AnyComponent>(&self, entity: &Entity) -> Option<WriteGuard<T>> {
        let mut component = self.world.entities().get_component_mut::<T>(entity)?;
        let change_count = self
            .world
            .entities()
            .storage::<T>()?
            .get_change_marker(entity)?;

        component.with_change_detection(change_count, self.world.change_tick());

        Some(component)
    }

    #[inline]
    pub fn insert_component<T: AnyComponent>(&self, entity: &Entity, component: T) {
        self.commands.insert_component(entity, component);
    }

    #[inline]
    pub fn remove_component<T: AnyComponent>(&self, entity: &Entity) {
        self.commands.remove_component::<T>(entity);
    }

    #[inline]
    pub fn get_resource<T: Resource>(&self) -> Option<ReadGuard<T>> {
        self.world.read_resource()
    }

    #[inline]
    pub fn get_resource_mut<T: Resource>(&self) -> Option<WriteGuard<T>> {
        self.world.write_resource()
    }

    #[inline]
    pub fn insert_resource<T: Resource>(&self, resource: T) {
        self.commands.insert_resource(resource);
    }

    #[inline]
    pub fn init_resource<T: Resource + Default>(&self) {
        self.commands.init_resource::<T>();
    }

    #[inline]
    pub fn remove_resource<T: Resource>(&self) {
        self.commands.remove_resource::<T>();
    }
}
