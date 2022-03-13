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

    pub fn commands(&self) -> &'s Commands<'w, 's> {
        self.commands
    }

    pub fn has_component<T: Component>(&self) -> bool {
        self.world()
            .entities()
            .contains_component::<T>(&self.entity)
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
    pub fn insert<T: Component>(&self, component: T) {
        self.commands().insert(&self.entity(), component);
    }

    #[track_caller]
    pub fn remove<T: Component>(&self) {
        self.commands().remove::<T>(&self.entity());
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

    #[track_caller]
    pub fn node(&self, entity: &Entity) -> Node<'w, 's> {
        if !self.world.entities().entities().contains(entity) {
            panic!("invalid entity: {}", entity)
        }

        Node::new(*entity, self.world, self.commands)
    }

    pub fn get_node(&self, entity: &Entity) -> Option<Node<'w, 's>> {
        if self.world.entities().entities().contains(entity) {
            Some(Node::new(*entity, self.world, self.commands))
        } else {
            None
        }
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

#[cfg(feature = "physics")]
impl<'w, 's> Node<'w, 's> {
    /// Casts a [`Ray`](ike_physics::Ray) from `position` to `direction` excluding intersections with `self`.
    pub fn cast_ray(
        &self,
        position: ike_math::Vec3,
        direction: ike_math::Vec3,
    ) -> Option<ike_physics::RayHit> {
        let ray_cast = ike_physics::RayCaster::new(self.world());
        ray_cast.cast_ray_exclude(position, direction, self.entity())
    }

    /// Casts a [`Ray`](ike_physics::Ray) from `position` to `direction` with a
    /// max `length` excluding intersections with `self`.
    pub fn cast_ray_length(
        &self,
        position: ike_math::Vec3,
        direction: ike_math::Vec3,
        length: f32,
    ) -> Option<ike_physics::RayHit> {
        let ray_cast = ike_physics::RayCaster::new(self.world());
        ray_cast.cast_ray_length_exclude(position, direction, length, self.entity())
    }
}
