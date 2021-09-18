use std::collections::HashMap;

use bytemuck::bytes_of;
use glam::Mat4;

use crate::{
    d3::{SampleOutput, TransformMaterial},
    id::HasId,
    prelude::{Color, DebugLine, UpdateCtx},
    renderer::{Drawable, RenderCtx},
};

use super::{Animation, D3Node, Mesh, PbrMaterial, Skeleton, Transform3d};

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
    pub animations: Vec<Animation>,
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
    pub animations: &'a [Animation],
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
    pub fn joint_matrices(&self) -> Vec<Mat4> {
        #[inline]
        fn joint_matrix(
            nodes: &HashMap<usize, PosedPbrNode>,
            matrices: &mut [Mat4],
            idx: &usize,
            transform: &Mat4,
        ) {
            let node = &nodes[idx];

            let matrix = *transform * node.transform.matrix();

            matrices[*idx] = matrix;

            for idx in node.children {
                joint_matrix(nodes, matrices, idx, &matrix);
            }
        }

        let mut joint_matrices = vec![Mat4::IDENTITY; self.nodes.len()];

        for idx in self.root_nodes {
            joint_matrix(
                &self.nodes,
                &mut joint_matrices,
                idx,
                &self.transform.matrix(),
            );
        }

        joint_matrices
    }

    #[inline]
    pub fn animate(&mut self, animation: usize, time: f32) {
        let animation = &self.animations[animation];

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
    }
}

impl Drawable for PosedPbrScene<'_> {
    type Node = D3Node;

    #[inline]
    fn draw(&self, ctx: &RenderCtx, node: &mut Self::Node) {
        let joint_matrices = self.joint_matrices();

        let inverse_global_transform = self.transform.matrix().inverse();

        for skeleton in self.skeletons.values() {
            let matrices = skeleton.joint_matrices(inverse_global_transform, &joint_matrices);

            let joint_matrices = node.meshes.joint_matrices.entry(skeleton.id()).or_default();

            let len = joint_matrices.joint_matrices.len();
            joint_matrices
                .joint_matrices
                .resize(len + matrices.len(), Mat4::IDENTITY);
            joint_matrices.joint_matrices[len..].copy_from_slice(&matrices);
        } 

        #[inline]
        fn draw_node(
            ctx: &RenderCtx,
            node: &PosedPbrNode,
            nodes: &HashMap<usize, PosedPbrNode>,
            d3_node: &mut D3Node,
            transform: &Transform3d,
            skeletons: &HashMap<usize, Skeleton>,
        ) {
            let transform = transform * &node.transform;

            if let Some(ref skeleton) = node.skeleton {
                for mesh in node.meshes {
                    d3_node.meshes.add_instance(
                        ctx,
                        &mesh.mesh,
                        bytes_of(&TransformMaterial {
                            joint_count: skeletons[skeleton].joints.len() as u32,
                            ..TransformMaterial::new(transform.matrix(), &mesh.material, 1 << 1)
                        }),
                        mesh.material.filter_mode,
                        Some(skeletons[skeleton].id()),
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
                        bytes_of(&TransformMaterial::new(transform.matrix(), &mesh.material, 0)),
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

            for idx in node.children {
                let node = &nodes[idx];

                draw_node(ctx, node, nodes, d3_node, &transform, skeletons);
            }
        }

        for idx in self.root_nodes {
            draw_node(
                ctx,
                &self.nodes[idx],
                &self.nodes,
                node,
                &self.transform,
                self.skeletons,
            );
        }
    }
}
