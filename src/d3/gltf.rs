use std::{borrow::Cow, collections::HashMap};

use glam::{Mat4, Quat, Vec2, Vec3};
use gltf::{
    animation::util::ReadOutputs,
    buffer,
    image::{self, Format},
    import, Node,
};

use crate::{
    id::Id,
    prelude::{Color, Color8, Texture, Vertex},
};

use super::{
    Animation, AnimationChannel, AnimationProperty, AnimationSample, AnimationSampler, Animations,
    ChannelTarget, Interpolation, Mesh, PbrMaterial, PbrMesh, PbrNode, PbrScene, SampleInput,
    SampleOutput, Skeleton, Transform3d,
};

impl AnimationChannel {
    #[inline]
    fn load_gltf(
        channel: &gltf::animation::Channel,
        samplers: &mut Vec<AnimationSampler>,
        buffers: &[buffer::Data],
    ) -> Self {
        let property = match channel.target().property() {
            gltf::animation::Property::Translation => AnimationProperty::Translation,
            gltf::animation::Property::Rotation => AnimationProperty::Rotation,
            gltf::animation::Property::Scale => AnimationProperty::Scale,
            gltf::animation::Property::MorphTargetWeights => AnimationProperty::MorphTargetWeights,
        };

        let target = ChannelTarget {
            node: channel.target().node().index(),
            property,
        };

        let sampler_index = samplers.len();

        let reader = channel.reader(|buffer| Some(&buffers[buffer.index()]));

        let samples = match reader.read_outputs().unwrap() {
            ReadOutputs::Translations(translations) => translations
                .zip(reader.read_inputs().unwrap())
                .map(|(p, input)| {
                    (
                        SampleInput(input),
                        AnimationSample {
                            input,
                            output: SampleOutput::Translation(p.into()),
                        },
                    )
                })
                .collect(),
            ReadOutputs::Rotations(rotations) => rotations
                .into_f32()
                .zip(reader.read_inputs().unwrap())
                .map(|(p, input)| {
                    (
                        SampleInput(input),
                        AnimationSample {
                            input,
                            output: SampleOutput::Rotation(Quat::from_xyzw(p[0], p[1], p[2], p[3])),
                        },
                    )
                })
                .collect(),
            ReadOutputs::Scales(scales) => scales
                .zip(reader.read_inputs().unwrap())
                .map(|(p, input)| {
                    (
                        SampleInput(input),
                        AnimationSample {
                            input,
                            output: SampleOutput::Scale(p.into()),
                        },
                    )
                })
                .collect(),
            ReadOutputs::MorphTargetWeights(weights) => weights
                .into_f32()
                .zip(reader.read_inputs().unwrap())
                .map(|(p, input)| {
                    (
                        SampleInput(input),
                        AnimationSample {
                            input,
                            output: SampleOutput::MorphTargetWeight(p),
                        },
                    )
                })
                .collect(),
        };

        samplers.push(AnimationSampler {
            samples,
            interpolation: match channel.sampler().interpolation() {
                gltf::animation::Interpolation::Linear => Interpolation::Linear,
                gltf::animation::Interpolation::Step => Interpolation::Step,
                gltf::animation::Interpolation::CubicSpline => Interpolation::CubicSpline,
            },
        });

        Self {
            target,
            sampler: sampler_index,
        }
    }
}

impl Animation {
    #[inline]
    fn load_gltf(animation: &gltf::Animation, buffers: &[buffer::Data]) -> Self {
        let mut channels = Vec::new();
        let mut samplers = Vec::new();

        for channel in animation.channels() {
            let channel = AnimationChannel::load_gltf(&channel, &mut samplers, buffers);

            channels.push(channel);
        }

        Self {
            name: animation.name().map(String::from),
            channels,
            samplers,
        }
    }
}

impl PbrNode {
    #[inline]
    fn load_gltf(
        node: &Node,
        nodes: &mut HashMap<usize, PbrNode>,
        skeletons: &mut HashMap<usize, Skeleton>,
        buffers: &[buffer::Data],
        images: &[image::Data],
    ) -> Self {
        let mut meshes = Vec::new();

        if let Some(ref node_mesh) = node.mesh() {
            for primitive in node_mesh.primitives() {
                let mut mesh = Mesh::new();

                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                let positions = reader
                    .read_positions()
                    .unwrap()
                    .map(|v| Vec3::from(v))
                    .collect::<Vec<_>>();

                let len = positions.len();

                let normals = if let Some(normals) = reader.read_normals() {
                    normals.map(|v| Vec3::from(v)).collect()
                } else {
                    vec![Vec3::Z; len]
                };

                let tangents = if let Some(tangents) = reader.read_tangents() {
                    tangents.map(|v| Vec3::new(v[0], v[1], v[2])).collect()
                } else {
                    vec![Vec3::X; len]
                };

                let uvs = if let Some(uvs) = reader.read_tex_coords(0) {
                    uvs.into_f32().map(|v| Vec2::from(v)).collect()
                } else {
                    vec![Vec2::ZERO; len]
                };

                let colors = if let Some(colors) = reader.read_colors(0) {
                    colors.into_rgba_f32().map(|v| Color::from(v)).collect()
                } else {
                    vec![Color::WHITE; len]
                };

                let joints = if let Some(joints) = reader.read_joints(0) {
                    joints
                        .into_u16()
                        .map(|[x, y, z, w]| [x as u32, y as u32, z as u32, w as u32])
                        .collect()
                } else {
                    vec![[0; 4]; len]
                };

                let weights = if let Some(weights) = reader.read_weights(0) {
                    weights.into_f32().collect()
                } else {
                    vec![[0.0; 4]; len]
                };

                for i in 0..len {
                    mesh.vertices.push(Vertex {
                        position: positions[i],
                        normal: normals[i],
                        tangent: tangents[i],
                        bitangent: normals[i].cross(tangents[i]),
                        uv: uvs[i],
                        color: colors[i],
                        joints: joints[i],
                        weights: weights[i],
                    });
                }

                if let Some(indices) = reader.read_indices() {
                    for index in indices.into_u32() {
                        mesh.indices.push(index);
                    }
                }

                let material = primitive.material();

                let pbr = material.pbr_metallic_roughness();

                let albedo_texture = pbr
                    .base_color_texture()
                    .map(|info| Cow::Owned(texture(&images[info.texture().index()])));

                let metallic_roughness_texture = pbr
                    .metallic_roughness_texture()
                    .map(|info| Cow::Owned(texture(&images[info.texture().index()])));

                let normal_map = material
                    .normal_texture()
                    .map(|info| Cow::Owned(texture(&images[info.texture().index()])));

                let emissive = material.emissive_factor();

                meshes.push(PbrMesh {
                    mesh,
                    material: PbrMaterial {
                        albedo_texture,
                        metallic_roughness_texture,
                        normal_map,
                        albedo: pbr.base_color_factor().into(),
                        roughness: pbr.roughness_factor(),
                        metallic: pbr.metallic_factor(),
                        emission: Color::rgba(emissive[0], emissive[1], emissive[2], 1.0),
                        ..Default::default()
                    },
                });
            }
        }

        let (translation, rotation, scale) = node.transform().decomposed();

        let mut children = Vec::new();

        for child in node.children() {
            children.push(child.index());

            let pbr_node = PbrNode::load_gltf(&child, nodes, skeletons, buffers, images);
            nodes.insert(child.index(), pbr_node);
        }

        let skeleton = node.skin().map(|skin| {
            let joints = skin.joints().map(|node| node.index()).collect();

            let reader = skin.reader(|buffer| Some(&buffers[buffer.index()]));

            let skeleton = Skeleton {
                id: Id::new(),
                inverse_bind_matrices: reader
                    .read_inverse_bind_matrices()
                    .unwrap()
                    .map(|m| Mat4::from_cols_array_2d(&m))
                    .collect(),
                joints,
                root: skin.joints().next().unwrap().index(),
            };

            skeletons.insert(skin.index(), skeleton);

            skin.index()
        });

        PbrNode {
            transform: Transform3d {
                translation: translation.into(),
                rotation: Quat::from_xyzw(rotation[0], rotation[1], rotation[2], rotation[3]),
                scale: scale.into(),
            },
            skeleton,
            meshes,
            children,
        }
    }
}

impl PbrScene {
    #[inline]
    pub fn load_gltf(path: impl AsRef<str>) -> anyhow::Result<Self> {
        let (gltf, buffers, images) = import(path.as_ref())?;

        let mut root_nodes = Vec::new();
        let mut nodes = HashMap::new();

        let mut skeletons = HashMap::new();

        for scene in gltf.scenes() {
            for node in scene.nodes() {
                let pbr_node =
                    PbrNode::load_gltf(&node, &mut nodes, &mut skeletons, &buffers, &images);
                nodes.insert(node.index(), pbr_node);
                root_nodes.push(node.index());
            }
        }

        let mut animations = Vec::new();

        for animation in gltf.animations() {
            animations.push(Animation::load_gltf(&animation, &buffers));
        }

        Ok(PbrScene {
            root_nodes,
            nodes,
            skeletons,
            animations: Animations { animations },
        })
    }
}

#[inline]
fn texture(data: &image::Data) -> Texture {
    let image_data = match data.format {
        Format::R8 => data
            .pixels
            .chunks(1)
            .map(|p| Color8::rgba(p[0], p[0], p[0], p[0]))
            .collect(),
        Format::R8G8 => data
            .pixels
            .chunks(2)
            .map(|p| Color8::rgba(p[0], p[1], 0, 255))
            .collect(),
        Format::R8G8B8 => data
            .pixels
            .chunks(3)
            .map(|p| Color8::rgba(p[0], p[1], p[2], 255))
            .collect(),
        Format::R8G8B8A8 => data
            .pixels
            .chunks(4)
            .map(|p| Color8::rgba(p[0], p[1], p[2], p[3]))
            .collect(),
        _ => unimplemented!("texture format not supported"),
    };

    Texture::from_data(image_data, data.width, data.height)
}
