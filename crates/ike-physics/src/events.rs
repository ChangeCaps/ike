use ike_ecs::world::Entity;
use ike_math::Vec3;

#[derive(Clone, Copy, Debug)]
pub struct CollisionData {
    pub rigid_body: Entity,
    pub collider: Entity,
    pub impulse: f32,
    pub normal: Vec3,
}

#[derive(Clone, Copy, Debug)]
pub struct Collision {
    pub started: bool,
    pub a: CollisionData,
    pub b: CollisionData,
}
