use crate::{AnyComponent, Node, World};

pub trait Component: AnyComponent {
    fn update(&mut self, node: &mut Node<'_>, world: &World);
}
