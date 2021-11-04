use crate::{AnyComponent, Node, World};

#[allow(unused)]
pub trait Component: AnyComponent {
    fn update(&mut self, node: &mut Node<'_>, world: &World) {}
}
