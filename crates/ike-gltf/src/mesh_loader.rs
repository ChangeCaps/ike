use ike_assets::{AssetLoader, Handle, LoadContext, LoadedAsset};
use ike_math::Quat;
use ike_render::{Image, Mesh};
use ike_transform::Transform;
use ike_util::BoxedFuture;

use crate::{GltfMesh, GltfMeshNode, GltfPrimitive};

pub struct GltfMeshLoader;

impl AssetLoader for GltfMeshLoader {
    fn load<'a>(
        &'a self,
        load_context: &'a mut LoadContext<'a>,
    ) -> BoxedFuture<'a, Result<(), ike_util::Error>> {
        Box::pin(async {
            let (document, buffers, images) = gltf::import_slice(load_context.bytes())?;

            let mut context = GltfLoadContext::new(&document, &buffers, &images, load_context);
            context.load_all();
            let mesh = context.load_gltf_mesh();
            load_context.set_default_asset(LoadedAsset::new(mesh));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["mesh.gltf", "mesh.glb"]
    }
}

struct GltfLoadContext<'a, 'b> {
    document: &'a gltf::Document,
    buffers: &'a [gltf::buffer::Data],
    images: &'a [gltf::image::Data],
    load_context: &'a mut LoadContext<'b>,
    #[cfg(feature = "pbr")]
    materials: Vec<Handle<ike_pbr::PbrMaterial>>,
    textures: Vec<Handle<Image>>,
    meshes: Vec<Vec<GltfPrimitive>>,
}

impl<'a, 'b> GltfLoadContext<'a, 'b> {
    pub fn new(
        document: &'a gltf::Document,
        buffers: &'a [gltf::buffer::Data],
        images: &'a [gltf::image::Data],
        load_context: &'a mut LoadContext<'b>,
    ) -> Self {
        Self {
            document,
            buffers,
            images,
            load_context,
            #[cfg(feature = "pbr")]
            materials: Vec::new(),
            textures: Vec::new(),
            meshes: Vec::new(),
        }
    }

    pub fn load_all(&mut self) {
        self.load_textures();
        #[cfg(feature = "pbr")]
        self.load_materials();
        self.load_primitives();
    }

    pub fn load_gltf_mesh(&self) -> GltfMesh {
        let mut nodes = Vec::new();

        for scene in self.document.scenes() {
            for node in scene.nodes() {
                nodes.push(self.load_gltf_mesh_node(node));
            }
        }

        GltfMesh { nodes }
    }

    pub fn load_gltf_mesh_node(&self, node: gltf::Node) -> GltfMeshNode {
        let children = node
            .children()
            .map(|node| self.load_gltf_mesh_node(node))
            .collect();

        let (translation, rotation, scale) = node.transform().decomposed();

        GltfMeshNode {
            transform: Transform {
                translation: translation.into(),
                rotation: Quat::from_array(rotation),
                scale: scale.into(),
            },
            primitives: node
                .mesh()
                .map_or(Vec::new(), |mesh| self.meshes[mesh.index()].clone()),
            children,
        }
    }

    #[cfg(feature = "pbr")]
    pub fn load_materials(&mut self) {
        for material in self.document.materials() {
            let pbr = material.pbr_metallic_roughness();

            let material = ike_pbr::PbrMaterial {
                base_color: pbr.base_color_factor().into(),
                base_color_texture: pbr
                    .base_color_texture()
                    .map(|info| self.textures[info.texture().index()].clone()),
                metallic: pbr.metallic_factor(),
                roughness: pbr.roughness_factor(),
                metallic_roughness_texture: pbr
                    .metallic_roughness_texture()
                    .map(|info| self.textures[info.texture().index()].clone()),
                emission: material.emissive_factor().into(),
                emission_texture: material
                    .emissive_texture()
                    .map(|info| self.textures[info.texture().index()].clone()),
                reflectance: 0.5,
                normal_map: material
                    .normal_texture()
                    .map(|texture| self.textures[texture.texture().index()].clone()),
            };

            let handle = self.load_context.add_asset(LoadedAsset::new(material));
            self.materials.push(handle);
        }
    }

    pub fn load_textures(&mut self) {
        for gltf_texture in self.document.textures() {
            let image = &self.images[gltf_texture.source().index()];
            let texture = Image::new_2d(image.pixels.clone(), image.width, image.height);
            let handle = self.load_context.add_asset(LoadedAsset::new(texture));
            self.textures.push(handle);
        }
    }

    pub fn load_primitives(&mut self) {
        for mesh in self.document.meshes() {
            let mut primitives = Vec::new();

            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| Some(&self.buffers[buffer.index()]));

                let mut mesh = Mesh::new();

                if let Some(positions) = reader.read_positions() {
                    mesh.insert_attribute(Mesh::POSITION, positions.collect());
                }

                if let Some(normals) = reader.read_normals() {
                    mesh.insert_attribute(Mesh::NORMAL, normals.collect());
                }

                if let Some(tangents) = reader.read_tangents() {
                    mesh.insert_attribute(Mesh::TANGENT, tangents.collect());
                }

                if let Some(uvs) = reader.read_tex_coords(0) {
                    mesh.insert_attribute(Mesh::UV_0, uvs.into_f32().collect());
                }

                if let Some(colors) = reader.read_colors(0) {
                    mesh.insert_attribute(Mesh::COLOR_0, colors.into_rgba_f32().collect());
                }

                if let Some(indices) = reader.read_indices() {
                    *mesh.get_indices_mut() = indices.into_u32().collect();
                }

                if !mesh.contains_attribute(Mesh::TANGENT) {
                    mesh.calculate_tangents();
                }

                let handle = self.load_context.add_asset(LoadedAsset::new(mesh));

                primitives.push(GltfPrimitive {
                    mesh: handle,
                    #[cfg(feature = "pbr")]
                    material: self.materials[primitive.material().index().unwrap()].clone(),
                });
            }

            self.meshes.push(primitives);
        }
    }
}
