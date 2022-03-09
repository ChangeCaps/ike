use ike_ecs::Entity;

#[derive(Clone, Debug)]
pub enum Collision {
    Started(Entity, Entity),
    Stopped(Entity, Entity),
}
