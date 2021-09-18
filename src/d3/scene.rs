use std::collections::HashMap;

use bytemuck::bytes_of;
use glam::Mat4;

use crate::{
    d3::{InstanceData, SampleOutput},
    id::HasId,
    prelude::{Color, DebugLine, UpdateCtx},
    renderer::{Drawable, RenderCtx},
};

use super::{
    Animation, AnimationError, AnimationIdent, Animations, D3Node, Mesh, PbrFlags, PbrMaterial,
    Skeleton, Transform3d,
};

#[derive(Clone, Debug)]
pub struct PbrMesh {
    pub mesh: Mesh,
    pub material: PbrMaterial<'static>,
}

#[derive(Clone)]
pub struct PbrNode {
    pub transform: Transform3d,
    pub skeleton: Option<usize>,
    pub meshes: Vec<PbrMesh>,
    pub children: Vec<usize>,
}

impl std::fmt::Debug for PbrNode {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PbrNode")
            .field("transform", &self.transform)
            .field("meshes", &self.meshes.len())
            .field("children", &self.children)
            .finish()
    }
}

#[derive(Clone, Debug)]
pub struct PbrScene {
    pub root_nodes: Vec<usize>,
    pub nodes: HashMap<usize, PbrNode>,
    pub skeletons: HashMap<usize, Skeleton>,
    pub animations: Animations,
}

impl PbrScene {
    #[inline]
    pub fn pose(&self) -> PosedPbrScene {
        let nodes = self
            .nodes
            .iter()
            .map(|(i, node)| {
                (
                    *i,
                    PosedPbrNode {
                        transform: Transform3d::IDENTITY,
                        skeleton: node.skeleton,
                        meshes: &node.meshes,
                        children: &node.children,
                    },
                )
            })
            .collect();

        PosedPbrScene {
            transform: Transform3d::IDENTITY,
            root_nodes: &self.root_nodes,
            nodes,
            skeletons: &self.skeletons,
            animations: &self.animations,
        }
    }

    #[inline]
    pub fn transform(&self, transform: &Transform3d) -> PosedPbrScene {
        let mut scene = self.pose();
        scene.transform = transform.clone();
        scene
    }
}

#[derive(Clone, Debug)]
pub struct PosedPbrNode<'a> {
    pub transform: Transform3d,
    pub skeleton: Option<usize>,
    pub meshes: &'a [PbrMesh],
    pub children: &'a [usize],
}

#[derive(Clone, Debug)]
pub struct PosedPbrScene<'a> {
    pub transform: Transform3d,
    pub root_nodes: &'a [usize],
    pub nodes: HashMap<usize, PosedPbrNode<'a>>,
    pub skeletons: &'a HashMap<usize, Skeleton>,
    pub animations: &'a Animations,
}

impl<'a> PosedPbrScene<'a> {
    #[inline]
    pub fn debug_skeletons(&self, ctx: &mut UpdateCtx) {
        fn debug_joint(
            nodes: &HashMap<usize, PosedPbrNode>,
            ctx: &mut UpdateCtx,
            idx: &usize,
            transform: &Transform3d,
            skeleton: &Skeleton,
        ) {
            let transform = transform * &nodes[idx].transform;
            let a = transform.translation;

            for idx in nodes[idx].children {
                let b = (&transform * &nodes[idx].transform).translation;

                ctx.draw(&DebugLine::color(a, b, Color::GREEN));

                debug_joint(nodes, ctx, idx, &transform, skeleton);
            }
        }

        for skeleton in self.skeletons.values() {
            debug_joint(&self.nodes, ctx, &skeleton.root, &self.transform, skeleton);
        }
    }

    #[inline]
    pub fn global_transforms(&self) -> Vec<Transform3d> {
        #[inline]
        fn global_transform(
            nodes: &HashMap<usize, PosedPbrNode>,
            matrices: &mut [Transform3d],
            idx: &usize,
            transform: &Transform3d,
        ) {
            let node = &nodes[idx];

            let node_transform = transform * &node.transform;

            for idx in node.children {
                global_transform(nodes, matrices, idx, &node_transform);
            }

            matrices[*idx] = node_transform;
        }

        let mut transforms = vec![Transform3d::IDENTITY; self.nodes.len()];

        for idx in self.root_nodes {
            global_transform(&self.nodes, &mut transforms, idx, &self.transform);
        }

        transforms
    }

    #[inline]
    pub fn animate<'b>(
        &'b mut self,
        animation: impl Into<AnimationIdent<'b>>,
        time: f32,
    ) -> Result<(), AnimationError> {
        let animation = self
            .animations
            .get(animation)
            .ok_or_else(|| AnimationError::AnimationNotFound)?; 

        for channel in &animation.channels {
            let sampler = &animation.samplers[channel.sampler];

            let output = if let Some(output) = sampler.sample(time) {
                output
            } else {
                continue;
            };

            let target = self.nodes.get_mut(&channel.target.node).unwrap();

            match output {
                SampleOutput::Translation(p) => target.transform.translation = p,
                SampleOutput::Rotation(p) => target.transform.rotation = p,
                SampleOutput::Scale(p) => target.transform.scale = p,
                _ => unimplemented!(),
            }
        }

        Ok(())
    }
}

impl Drawable for PosedPbrScene<'_> {
    type Node = D3Node;

    #[inline]
    fn draw(&self, ctx: &RenderCtx, d3_node: &mut Self::Node) {
        let global_transforms = self.global_transforms();

        let inverse_global_transform = self.transform.matrix().inverse();

        for skeleton in self.skeletons.values() {
            let mut matrices = skeleton.joint_matrices(inverse_global_transform, &global_transforms);

            let joint_matrices = d3_node
                .meshes
                .joint_matrices
                .entry(skeleton.id())
                .or_default();

            joint_matrices.joint_matrices.append(&mut matrices)
        }

        for (idx, node) in &self.nodes {
            if let Some(ref skeleton) = node.skeleton {
                let skeleton = &self.skeletons[skeleton];

                for mesh in node.meshes {
                    d3_node.meshes.add_instance(
                        ctx,
                        &mesh.mesh,
                        bytes_of(&InstanceData {
                            joint_count: skeleton.joints.len() as u32,
                            ..InstanceData::new(
                                global_transforms[*idx].matrix(),
                                &mesh.material,
                                PbrFlags::SKINNED,
                            )
                        }),
                        mesh.material.filter_mode,
                        Some(skeleton.id()),
                        mesh.material.albedo_texture.as_ref().map(AsRef::as_ref),
                        mesh.material
                            .metallic_roughness_texture
                            .as_ref()
                            .map(AsRef::as_ref),
                        mesh.material.normal_map.as_ref().map(AsRef::as_ref),
                    );
                }
            } else {
                for mesh in node.meshes {
                    d3_node.meshes.add_instance(
                        ctx,
                        &mesh.mesh,
                        bytes_of(&InstanceData::new(
                            global_transforms[*idx].matrix(),
                            &mesh.material,
                            PbrFlags::EMPTY,
                        )),
                        mesh.material.filter_mode,
                        None,
                        mesh.material.albedo_texture.as_ref().map(AsRef::as_ref),
                        mesh.material
                            .metallic_roughness_texture
                            .as_ref()
                            .map(AsRef::as_ref),
                        mesh.material.normal_map.as_ref().map(AsRef::as_ref),
                    );
                }
            }
        }
    }
}
