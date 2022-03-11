use bytemuck::{bytes_of, Pod, Zeroable};
use ike_assets::{AssetEvent, Assets, Handle};
use ike_ecs::{EventReader, Res, ResMut};
use ike_render::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindingResource, Buffer,
    BufferInitDescriptor, BufferUsages, Color, Image, ImageTexture, RenderDevice,
};

use crate::PbrResources;

pub struct PbrMaterial {
    pub base_color: Color,
    pub base_color_texture: Option<Handle<Image>>,
    pub metallic: f32,
    pub roughness: f32,
    pub metallic_roughness_texture: Option<Handle<Image>>,
    pub reflectance: f32,
    pub emission: Color,
    pub emission_texture: Option<Handle<Image>>,
    pub normal_map: Option<Handle<Image>>,
}

impl Default for PbrMaterial {
    fn default() -> Self {
        Self {
            base_color: Color::WHITE,
            base_color_texture: None,
            metallic: 0.1,
            roughness: 0.981,
            metallic_roughness_texture: None,
            reflectance: 0.5,
            emission: Color::BLACK,
            emission_texture: None,
            normal_map: None,
        }
    }
}

impl PbrMaterial {
    pub fn contains_image(&self, changed: &Handle<Image>) -> bool {
        self.base_color_texture.as_ref() == Some(changed)
            || self.metallic_roughness_texture.as_ref() == Some(changed)
            || self.normal_map.as_ref() == Some(changed)
            || self.emission_texture.as_ref() == Some(changed)
    }

    pub fn as_raw(&self) -> RawPbrMaterial {
        RawPbrMaterial {
            base_color: self.base_color.into(),
            metallic: self.metallic,
            roughness: self.roughness,
            reflectance: self.reflectance,
            _padding: [0; 4],
            emission: self.emission.into(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct RawPbrMaterial {
    pub base_color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub reflectance: f32,
    pub _padding: [u8; 4],
    pub emission: [f32; 4],
}

pub struct MaterialBinding {
    pub base_color: Option<Handle<Image>>,
    pub metallic_roughness: Option<Handle<Image>>,
    pub normal_map: Option<Handle<Image>>,
    pub emission: Option<Handle<Image>>,
    pub buffer: Buffer,
    pub bind_group: BindGroup,
}

impl MaterialBinding {
    pub fn get_texture<'a>(
        image: &Option<Handle<Image>>,
        default_image: &'a ImageTexture,
        image_textures: &'a Assets<ImageTexture>,
    ) -> &'a ImageTexture {
        image
            .as_ref()
            .and_then(|image| image_textures.get(image))
            .unwrap_or(default_image)
    }

    pub fn system(
        mut image_events: EventReader<AssetEvent<ImageTexture>>,
        mut material_events: EventReader<AssetEvent<PbrMaterial>>,
        device: Res<RenderDevice>,
        pbr_resources: Res<PbrResources>,
        materials: Res<Assets<PbrMaterial>>,
        image_textures: Res<Assets<ImageTexture>>,
        mut material_bindings: ResMut<Assets<MaterialBinding>>,
    ) {
        for material_event in material_events.iter() {
            match material_event {
                AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                    let material = materials.get(handle).unwrap();

                    material_bindings.insert(
                        handle,
                        MaterialBinding::new(
                            material,
                            &device,
                            &pbr_resources.material_bind_group_layout,
                            &pbr_resources.default_image,
                            &pbr_resources.default_normal_map,
                            &image_textures,
                        ),
                    );
                }
                _ => {}
            }
        }

        for image_event in image_events.iter() {
            match image_event {
                AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                    // TODO: this is very inefficient
                    for (&material_handle, material) in materials.iter() {
                        if material.contains_image(&handle.cast()) {
                            material_bindings.insert(
                                material_handle,
                                MaterialBinding::new(
                                    material,
                                    &device,
                                    &pbr_resources.material_bind_group_layout,
                                    &pbr_resources.default_image,
                                    &pbr_resources.default_normal_map,
                                    &image_textures,
                                ),
                            );
                        }
                    }
                }
                _ => {}
            }
        }
    }

    pub fn new(
        material: &PbrMaterial,
        device: &RenderDevice,
        layout: &BindGroupLayout,
        default_image: &ImageTexture,
        default_normal_map: &ImageTexture,
        image_textures: &Assets<ImageTexture>,
    ) -> Self {
        let raw_material = material.as_raw();

        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("pbr_material_buffer"),
            contents: bytes_of(&raw_material),
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
        });

        let base_color =
            Self::get_texture(&material.base_color_texture, default_image, image_textures);

        let metallic_roughness = Self::get_texture(
            &material.metallic_roughness_texture,
            default_image,
            image_textures,
        );
        let emission = Self::get_texture(&material.emission_texture, default_image, image_textures);
        let normal_map = material
            .normal_map
            .as_ref()
            .and_then(|image| image_textures.get(image))
            .unwrap_or(default_normal_map);

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("pbr_material_bind_group"),
            layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: buffer.raw().as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(base_color.texture_view.raw()),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(&base_color.sampler),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::TextureView(metallic_roughness.texture_view.raw()),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: BindingResource::Sampler(&metallic_roughness.sampler),
                },
                BindGroupEntry {
                    binding: 5,
                    resource: BindingResource::TextureView(emission.texture_view.raw()),
                },
                BindGroupEntry {
                    binding: 6,
                    resource: BindingResource::Sampler(&emission.sampler),
                },
                BindGroupEntry {
                    binding: 7,
                    resource: BindingResource::TextureView(normal_map.texture_view.raw()),
                },
                BindGroupEntry {
                    binding: 8,
                    resource: BindingResource::Sampler(&normal_map.sampler),
                },
            ],
        });

        Self {
            base_color: material.base_color_texture.clone(),
            metallic_roughness: material.metallic_roughness_texture.clone(),
            normal_map: material.normal_map.clone(),
            emission: material.emission_texture.clone(),
            buffer,
            bind_group,
        }
    }
}
