use ike_core::Node;

use crate::Parent;

pub trait TransformNodeExt {
    fn get_parent(&self) -> Option<Node>;
}

impl<'a> TransformNodeExt for Node<'a> {
    #[inline]
    fn get_parent(&self) -> Option<Node> {
        let parent = self.get_component::<Parent>()?;

        self.world().get_node(parent.0)
    }
}
