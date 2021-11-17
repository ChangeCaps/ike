use bytemuck::cast_slice;
use glam::Quat;
use gltf::{
    buffer,
    image::{self, Format},
    Node,
};
use ike_assets::Assets;
use ike_core::{Entity, WorldRef};
use ike_pbr::PbrMaterial;
use ike_render::{Color8, ColorSpace, Mesh, Texture};
use ike_scene::{Scene, SceneNode};
use ike_transform::{Parent, Transform};
use std::path::Path;

pub fn load_gltf(path: impl AsRef<Path>, world: &WorldRef) -> Result<Scene, gltf::Error> {
    let (document, buffers, images) = gltf::import(path)?;

    let mut scene = Scene::new();

    for gltf_scene in document.scenes() {
        for gltf_node in gltf_scene.nodes() {
            load_node(&mut scene, &gltf_node, &None, world, &buffers, &images);
        }
    }

    Ok(scene)
}

fn load_node(
    scene: &mut Scene,
    gltf_node: &Node,
    parent: &Option<Entity>,
    world: &WorldRef,
    buffers: &[buffer::Data],
    images: &[image::Data],
) {
    let name = gltf_node.name().unwrap_or("gltf_nope");
    let mut node = SceneNode::new(name);

    let (translation, rotation, scale) = gltf_node.transform().decomposed();
    let mut transform = Transform::IDENTITY;
    transform.translation = translation.into();
    transform.rotation = Quat::from_xyzw(rotation[0], rotation[1], rotation[2], rotation[3]);
    transform.scale = scale.into();

    node.insert(transform);

    if let &Some(parent) = parent {
        node.insert(Parent(parent));
    }

    if let Some(mesh) = gltf_node.mesh() {
        load_mesh(&mut node, &mesh, world, buffers, images);
    }

    let parent = Some(scene.add(node));

    for gltf_node in gltf_node.children() {
        load_node(scene, &gltf_node, &parent, world, buffers, images)
    }
}

fn load_mesh(
    node: &mut SceneNode,
    gltf_mesh: &gltf::Mesh,
    world: &WorldRef,
    buffers: &[buffer::Data],
    images: &[image::Data],
) {
    let mut mesh = Mesh::new();

    if let Some(primitive) = gltf_mesh.primitives().next() {
        let reader =
            primitive.reader(|buffer| buffers.get(buffer.index()).map(|data| data.0.as_slice()));

        let mut vertex_count = 0;

        if let Some(positions) = reader.read_positions() {
            vertex_count = positions.len();
            mesh.insert(Mesh::POSITION, positions.collect());
        }

        if let Some(uvs) = reader.read_tex_coords(0) {
            let uvs: Vec<_> = uvs.into_f32().collect();
            vertex_count = uvs.len();
            mesh.insert(Mesh::UV, uvs);
        }

        if let Some(normals) = reader.read_normals() {
            vertex_count = normals.len();
            mesh.insert(Mesh::NORMAL, normals.collect());
        }

        if let Some(tangents) = reader.read_tangents() {
            vertex_count = tangents.len();
            mesh.insert(Mesh::TANGENT, tangents.collect());
        } else {
            mesh.insert(Mesh::TANGENT, vec![[0.0; 4]; vertex_count]);
        }

        if let Some(colors) = reader.read_colors(0) {
            mesh.insert(Mesh::COLOR, colors.into_rgba_f32().collect());
        } else {
            mesh.insert(Mesh::COLOR, vec![[1.0; 4]; vertex_count]);
        }

        if let Some(indices) = reader.read_indices() {
            *mesh.indices_mut() = indices.into_u32().collect();
        }

        let material = primitive.material();

        let metallic_roughness = material.pbr_metallic_roughness();

        let mut textures = world.get_resource_mut::<Assets<Texture>>().unwrap();

        let albedo_texture = metallic_roughness.base_color_texture().map(|albedo| {
            let texture = albedo.texture();
            let image = &images[texture.source().index()];

            let texture = Texture::from_data(
                cast_slice(&image.pixels).to_vec(),
                image.width,
                image.height,
            );

            textures.add(texture)
        });

        let metallic_roughness_texture =
            metallic_roughness
                .metallic_roughness_texture()
                .map(|metallic_roughness| {
                    let texture = metallic_roughness.texture();
                    let image = &images[texture.source().index()];

                    let texture = Texture::from_data(
                        cast_slice(&image.pixels).to_vec(),
                        image.width,
                        image.height,
                    );

                    textures.add(texture)
                });

        let normal_map = material.normal_texture().map(|normal| {
            let texture = normal.texture();
            let image = &images[texture.source().index()];

            let mut texture = Texture::from_data(
                convert_data(&image.pixels, image.format),
                image.width,
                image.height,
            );

            texture.set_color_space(ColorSpace::Linear);

            textures.add(texture)
        });

        let pbr_material = PbrMaterial {
            albedo_texture,
            metallic_roughness_texture,
            normal_map,
            albedo: metallic_roughness.base_color_factor().into(),
            roughness: metallic_roughness.roughness_factor(),
            metallic: metallic_roughness.metallic_factor(),
            ..Default::default()
        };

        let mut materials = world.get_resource_mut::<Assets<PbrMaterial>>().unwrap();

        let handle = materials.add(pbr_material);

        node.insert(handle);
    }

    let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();

    let handle = meshes.add(mesh);

    node.insert(handle);
}

#[inline]
fn convert_data(pixels: &[u8], format: Format) -> Vec<Color8> {
    match format {
        Format::R8 => pixels
            .chunks(1)
            .map(|data| Color8::rgba(data[0], data[0], data[0], 255))
            .collect(),
        Format::R8G8 => pixels
            .chunks(2)
            .map(|data| Color8::rgba(data[0], data[1], 0, 255))
            .collect(),
        Format::R8G8B8 => pixels
            .chunks(3)
            .map(|data| Color8::rgba(data[0], data[1], data[2], 255))
            .collect(),
        Format::R8G8B8A8 => pixels
            .chunks(4)
            .map(|data| Color8::rgba(data[0], data[1], data[2], data[3]))
            .collect(),
        _ => unimplemented!(),
    }
}
