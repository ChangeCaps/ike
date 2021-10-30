use crate::World;

pub trait System {
    fn update(&mut self, _world: &World) {}
}
