use bytemuck::{bytes_of, cast_slice, Pod, Zeroable};
use ike_math::Mat4;

use std::mem;

use crate::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferDescriptor, BufferUsages,
    RawCamera, RenderDevice, RenderQueue, ShaderStages,
};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct RawMeshBinding {
    view_proj: [[f32; 4]; 4],
    camera_position: [f32; 3],
    _padding: [u8; 4],
}

impl RawMeshBinding {
    pub fn new(camera: &RawCamera) -> Self {
        Self {
            view_proj: camera.view_proj().to_cols_array_2d(),
            camera_position: camera.position.into(),
            _padding: [0u8; 4],
        }
    }
}

pub struct MeshBinding {
    pub buffer_size: u64,
    pub instances: Vec<[[f32; 4]; 4]>,
    pub instance_buffer: Buffer,
    pub buffer: Buffer,
    pub bind_group: BindGroup,
}

impl MeshBinding {
    pub fn new(device: &RenderDevice) -> Self {
        let buffer_size = 64 * 4;

        let instance_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("ike_mesh_binding_buffer"),
            size: buffer_size,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let buffer = device.create_buffer(&BufferDescriptor {
            label: Some("ike_mesh_binding_buffer"),
            size: mem::size_of::<RawMeshBinding>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = Self::bind_group_layout(device);

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("ike_mesh_binding_bind_group"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.raw().as_entire_binding(),
            }],
        });

        Self {
            buffer_size,
            instances: Vec::new(),
            instance_buffer,
            buffer,
            bind_group,
        }
    }

    pub fn instances(&self) -> u32 {
        self.instances.len() as u32
    }

    pub fn clear(&mut self) {
        self.instances.clear();
    }

    pub fn bind_group_layout(device: &RenderDevice) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("ike_mesh_binding_bind_group_layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                visibility: ShaderStages::VERTEX_FRAGMENT,
                count: None,
            }],
        })
    }

    pub fn push_instance(&mut self, transform: Mat4) {
        self.instances.push(transform.to_cols_array_2d());
    }

    pub fn write(&mut self, device: &RenderDevice, queue: &RenderQueue, camera: &RawCamera) {
        let raw = RawMeshBinding::new(camera);
        queue.write_buffer(&self.buffer, 0, bytes_of(&raw));

        if self.buffer_size < self.instances.len() as u64 * 64 {
            self.buffer_size = self.instances.len() as u64 * 64;

            let instance_buffer = device.create_buffer(&BufferDescriptor {
                label: Some("ike_mesh_binding_buffer"),
                size: self.buffer_size,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            self.instance_buffer = instance_buffer;
        }

        queue.write_buffer(&self.instance_buffer, 0, cast_slice(&self.instances));
    }
}
