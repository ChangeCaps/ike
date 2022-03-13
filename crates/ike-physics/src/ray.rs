use ike_ecs::{Entity, Query, Res, SystemParam, World};
use ike_math::Vec3;
use rapier3d::{
    parry::query::Ray as RawRay,
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

#[derive(Clone, Default)]
pub struct Ray<'a> {
    pub position: Vec3,
    pub direction: Vec3,
    pub max_length: Option<f32>,
    pub filter: Option<&'a dyn Fn(Entity) -> bool>,
}

impl<'a> Ray<'a> {
    pub fn cast(&self, caster: &RayCaster) -> Option<RayHit> {
        let func = if let Some(filter) = self.filter {
            Some(|collider| {
                if let Some(&entity) = caster.colliders.0.get(&collider) {
                    filter(entity)
                } else {
                    false
                }
            })
        } else {
            None
        };

        let (collider, intersection) = caster.query_pipeline.cast_ray_and_get_normal(
            &*caster.collider_set,
            &RawRay::new(to_vec3(self.position).into(), to_vec3(self.direction)),
            self.max_length.unwrap_or(f32::MAX),
            true,
            InteractionGroups::all(),
            func.as_ref().map(|f| f as _),
        )?;

        let &entity = caster.colliders.0.get(&collider)?;

        if !caster.entities.contains(&entity) {
            return None;
        }

        let distance = intersection.toi;

        Some(RayHit {
            entity,
            distance,
            position: self.position + self.direction.normalize() * distance,
            normal: from_vec3(intersection.normal),
        })
    }
}

#[derive(SystemParam)]
pub struct RayCaster<'w> {
    query_pipeline: Res<'w, QueryPipeline>,
    collider_set: Res<'w, ColliderSet>,
    colliders: Res<'w, Colliders>,
    entities: Query<'w, Entity>,
}

impl<'w> RayCaster<'w> {
    pub fn new(world: &'w World) -> Self {
        Self {
            query_pipeline: world.resource(),
            collider_set: world.resource(),
            colliders: world.resource(),
            entities: world.query().unwrap(),
        }
    }

    pub fn cast_ray(&self, position: Vec3, direction: Vec3) -> Option<RayHit> {
        let ray = Ray {
            position,
            direction,
            ..Default::default()
        };

        ray.cast(self)
    }

    pub fn cast_ray_length(&self, position: Vec3, direction: Vec3, length: f32) -> Option<RayHit> {
        let ray = Ray {
            position,
            direction,
            max_length: Some(length),
            ..Default::default()
        };

        ray.cast(self)
    }

    pub fn cast_ray_exclude(
        &self,
        position: Vec3,
        direction: Vec3,
        exclude: &Entity,
    ) -> Option<RayHit> {
        let func = |entity| entity != *exclude;

        let ray = Ray {
            position,
            direction,
            filter: Some(&func),
            ..Default::default()
        };

        ray.cast(self)
    }

    pub fn cast_ray_length_exclude(
        &self,
        position: Vec3,
        direction: Vec3,
        length: f32,
        exclude: &Entity,
    ) -> Option<RayHit> {
        let func = |entity| entity != *exclude;

        let ray = Ray {
            position,
            direction,
            max_length: Some(length),
            filter: Some(&func),
            ..Default::default()
        };

        ray.cast(self)
    }
}
