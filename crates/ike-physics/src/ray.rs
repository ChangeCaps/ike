use ike_ecs::{Entity, Res, SystemParam, World};
use ike_math::Vec3;
use rapier3d::{
    parry::query::Ray,
    prelude::{ColliderSet, InteractionGroups, QueryPipeline},
};

use crate::{from_vec3, to_vec3, Colliders};

#[derive(Clone, Copy, Debug)]
pub struct RayHit {
    pub entity: Entity,
    pub distance: f32,
    pub position: Vec3,
    pub normal: Vec3,
}

#[derive(SystemParam)]
pub struct RayCast<'w> {
    query_pipeline: Res<'w, QueryPipeline>,
    collider_set: Res<'w, ColliderSet>,
    colliders: Res<'w, Colliders>,
}

impl<'w> RayCast<'w> {
    pub fn new(world: &'w World) -> Self {
        Self {
            query_pipeline: world.resource(),
            collider_set: world.resource(),
            colliders: world.resource(),
        }
    }

    pub fn cast_ray(&self, position: Vec3, direction: Vec3, length: Option<f32>) -> Option<RayHit> {
        let (collider, intersection) = self.query_pipeline.cast_ray_and_get_normal(
            &*self.collider_set,
            &Ray::new(to_vec3(position).into(), to_vec3(direction)),
            length.unwrap_or(f32::MAX),
            false,
            InteractionGroups::all(),
            None,
        )?;

        let &entity = self.colliders.0.get(&collider)?;

        let distance = intersection.toi;

        Some(RayHit {
            entity,
            distance,
            position: position + direction.normalize() * distance,
            normal: from_vec3(intersection.normal),
        })
    }
}
