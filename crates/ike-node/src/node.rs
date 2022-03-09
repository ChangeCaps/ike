use ike_ecs::{
    Children, Commands, Comp, CompMut, Component, Entity, Query, QueryFilter, Res, ResMut,
    Resource, World, WorldQuery,
};

pub struct Node<'w, 's> {
    entity: Entity,
    world: &'w World,
    commands: &'s Commands<'w, 's>,
}

impl<'w, 's> Node<'w, 's> {
    pub fn new(entity: Entity, world: &'w World, commands: &'s Commands<'w, 's>) -> Self {
        Self {
            entity,
            world,
            commands,
        }
    }

    pub fn world(&self) -> &'w World {
        self.world
    }

    #[track_caller]
    pub fn component<T: Component>(&self) -> Comp<'w, T> {
        self.world().component(&self.entity).unwrap()
    }

    #[track_caller]
    pub fn component_mut<T: Component>(&self) -> CompMut<'w, T> {
        self.world().component_mut(&self.entity).unwrap()
    }

    #[track_caller]
    pub fn resource<T: Resource>(&self) -> Res<'w, T> {
        self.world().resource()
    }

    #[track_caller]
    pub fn resource_mut<T: Resource>(&self) -> ResMut<'w, T> {
        self.world().resource_mut()
    }

    #[track_caller]
    pub fn query<Q: WorldQuery>(&self) -> Query<'w, Q> {
        self.world().query().expect("query not available")
    }

    #[track_caller]
    pub fn query_filter<Q: WorldQuery, F: QueryFilter>(&self) -> Query<'w, Q, F> {
        self.world().query_filter().expect("query not available")
    }

    #[track_caller]
    pub fn child(&self, index: usize) -> Node<'w, 's> {
        let children = self.component::<Children>();
        let entity = children[index];
        Node::new(entity, self.world, self.commands)
    }

    pub fn entity(&self) -> &Entity {
        &self.entity
    }

    pub fn despawn(self) {
        self.commands.despawn(&self.entity);
    }

    pub fn despawn_recursive(self) {
        self.commands.despawn_recursive(&self.entity);
    }
}
