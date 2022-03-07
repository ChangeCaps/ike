use bytemuck::{bytes_of, Pod, Zeroable};
use ike_math::Mat4;
use std::{mem, ops::Index};

use crate::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferDescriptor, BufferUsages,
    RawCamera, RenderDevice, RenderQueue, ShaderStages,
};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct RawMeshBinding {
    transform: [[f32; 4]; 4],
    view_proj: [[f32; 4]; 4],
    camera_position: [f32; 3],
    _padding: [u8; 4],
}

impl RawMeshBinding {
    pub fn new(transform: Mat4, camera: &RawCamera) -> Self {
        Self {
            transform: transform.to_cols_array_2d(),
            view_proj: camera.view_proj().to_cols_array_2d(),
            camera_position: camera.position.into(),
            _padding: [0u8; 4],
        }
    }
}

pub struct MeshBinding {
    pub buffer: Buffer,
    pub bind_group: BindGroup,
}

impl MeshBinding {
    pub fn new(device: &RenderDevice) -> Self {
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

        Self { buffer, bind_group }
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

    pub fn write(&self, queue: &RenderQueue, transform: Mat4, camera: &RawCamera) {
        let raw = RawMeshBinding::new(transform, camera);
        queue.write_buffer(&self.buffer, 0, bytes_of(&raw));
    }
}

#[derive(Default)]
pub struct MeshBindings {
    bindings: Vec<MeshBinding>,
}

impl MeshBindings {
    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    pub fn require(&mut self, len: usize, device: &RenderDevice) {
        self.bindings
            .resize_with(usize::max(len + 1, self.len()), || MeshBinding::new(device));
    }

    pub fn iter(&self) -> impl Iterator<Item = &MeshBinding> {
        self.bindings.iter()
    }
}

impl Index<usize> for MeshBindings {
    type Output = MeshBinding;

    fn index(&self, index: usize) -> &Self::Output {
        &self.bindings[index]
    }
}
