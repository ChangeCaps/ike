use bytemuck::{bytes_of, Pod, Zeroable};
use ike_ecs::{FromResources, Resources};
use ike_render::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferBindingType,
    BufferDescriptor, BufferUsages, CompareFunction, Extent3d, RenderDevice, RenderQueue, Sampler,
    SamplerBindingType, SamplerDescriptor, ShaderStages, Texture, TextureDescriptor,
    TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureViewDescriptor,
    TextureViewDimension,
};
use std::mem;

use crate::{RawDirectionalLight, DIRECTIONAL_LIGHT_SHADOW_MAP_SIZE, MAX_DIRECTIONAL_LIGHTS};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct RawLights {
    pub directional_light_count: u32,
    pub _padding: [u8; 12],
    pub directional_lights: [RawDirectionalLight; MAX_DIRECTIONAL_LIGHTS as usize],
}

impl RawLights {
    pub fn new() -> Self {
        Self::zeroed()
    }

    pub fn push_directional_light(&mut self, directional_light: RawDirectionalLight) {
        if self.directional_light_count + 1 < MAX_DIRECTIONAL_LIGHTS {
            self.directional_lights[self.directional_light_count as usize] = directional_light;
            self.directional_light_count += 1;
        }
    }
}

pub struct LightBindings {
    pub buffer: Buffer,
    pub directional_light_shadow_maps: Texture,
    pub sampler: Sampler,
    pub bind_group_layout: BindGroupLayout,
    pub bind_group: BindGroup,
}

impl LightBindings {
    pub fn write(&self, queue: &RenderQueue, raw_lights: &RawLights) {
        queue.write_buffer(&self.buffer, 0, bytes_of(raw_lights));
    }
}

impl FromResources for LightBindings {
    fn from_resources(resources: &Resources) -> Self {
        let device = resources.read::<RenderDevice>().unwrap();

        let buffer = device.create_buffer(&BufferDescriptor {
            label: Some("ike_lights_buffer"),
            size: mem::size_of::<RawLights>() as u64,
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        let directional_light_shadow_maps = device.create_texture(&TextureDescriptor {
            label: Some("ike_directional_light_shadow_maps_texture"),
            size: Extent3d {
                width: DIRECTIONAL_LIGHT_SHADOW_MAP_SIZE,
                height: DIRECTIONAL_LIGHT_SHADOW_MAP_SIZE,
                depth_or_array_layers: MAX_DIRECTIONAL_LIGHTS,
            },
            dimension: TextureDimension::D2,
            mip_level_count: 1,
            sample_count: 1,
            format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
        });

        let directional_light_shadow_maps_view =
            directional_light_shadow_maps.create_view(&TextureViewDescriptor {
                dimension: Some(TextureViewDimension::D2Array),
                ..Default::default()
            });

        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("ike_shadow_map_sampler"),
            compare: Some(CompareFunction::LessEqual),
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("ike_lights_bind_group_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Depth,
                        view_dimension: TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    ty: BindingType::Sampler(SamplerBindingType::Comparison),
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("ike_lights_bind_group"),
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: buffer.raw().as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(
                        directional_light_shadow_maps_view.raw(),
                    ),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
        });

        Self {
            buffer,
            directional_light_shadow_maps,
            sampler,
            bind_group_layout,
            bind_group,
        }
    }
}
