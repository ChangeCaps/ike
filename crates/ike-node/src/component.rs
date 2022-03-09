use ike_ecs::{CompMut, Component};

use crate::Node;

pub trait NodeComponent: Component {
    fn stages() -> &'static [NodeFn<Self>];
}

pub struct NodeFn<T> {
    pub name: &'static str,
    pub func: fn(CompMut<T>, Node),
}
