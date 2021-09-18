use std::collections::HashMap;

use glam::Mat4;

use crate::id::{HasId, Id};

use super::Transform3d;

#[derive(Clone, Debug)]
pub struct Skeleton {
    pub(crate) id: Id<Skeleton>,
    pub inverse_bind_matrices: Vec<Mat4>,
    pub joints: Vec<usize>,
    pub root: usize,
}

impl HasId<Skeleton> for Skeleton {
    #[inline]
    fn id(&self) -> Id<Skeleton> {
        self.id
    }
}

impl Skeleton {
    #[inline]
    pub fn joint_matrices(
        &self,
        inverse_global_transform: Mat4,
        global_node_matrices: &[Transform3d],
    ) -> Vec<Mat4> {
        self.joints
            .iter()
            .enumerate()
            .map(|(bind, joint)| {
                inverse_global_transform
                    * global_node_matrices[*joint].matrix()
                    * self.inverse_bind_matrices[bind]
            })
            .collect()
    }
}
