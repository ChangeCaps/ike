use std::collections::HashMap;

use bytemuck::{cast_slice, cast_vec};
use glam::{Vec2, Vec3, Vec4};

use crate::{Buffer, Color};

#[derive(Clone, Debug)]
pub enum VertexData {
    Float32(Vec<f32>),
    Float32x2(Vec<[f32; 2]>),
    Float32x3(Vec<[f32; 3]>),
    Float32x4(Vec<[f32; 4]>),
}

impl VertexData {
    #[inline]
    pub fn data(&self) -> &[u8] {
        match self {
            VertexData::Float32(data) => cast_slice(data),
            VertexData::Float32x2(data) => cast_slice(data),
            VertexData::Float32x3(data) => cast_slice(data),
            VertexData::Float32x4(data) => cast_slice(data),
        }
    }
}

#[derive(Clone)]
pub struct VertexAttribute {
    update: bool,
    buffer: Buffer,
    data: VertexData,
}

impl VertexAttribute {
    #[inline]
    pub fn new(data: VertexData) -> Self {
        Self {
            update: true,
            buffer: Buffer::new(ike_wgpu::BufferUsages::VERTEX),
            data,
        }
    }
}

#[derive(Clone)]
pub struct Mesh {
    vertices: HashMap<String, VertexAttribute>,
    update_index_buffer: bool,
    indices: Vec<u32>,
    index_buffer: Buffer,
}

impl Default for Mesh {
    fn default() -> Self {
        Self {
            vertices: HashMap::new(),
            update_index_buffer: true,
            indices: Vec::new(),
            index_buffer: Buffer::new(ike_wgpu::BufferUsages::INDEX),
        }
    }
}

impl Mesh {
    pub const POSITION: &'static str = "position";
    pub const NORMAL: &'static str = "normal";
    pub const TANGENT: &'static str = "tangent";
    pub const UV: &'static str = "uv";
    pub const COLOR: &'static str = "color";
    pub const JOINT_INDICES: &'static str = "joint_indices";
    pub const JOINT_WEIGHTS: &'static str = "joint_weights";

    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn insert<T: IntoVertexData>(&mut self, name: impl Into<String>, data: Vec<T>) {
        self.vertices
            .insert(name.into(), VertexAttribute::new(T::into_data(data)));
    }

    #[inline]
    pub fn indices(&self) -> &Vec<u32> {
        &self.indices
    }

    #[inline]
    pub fn indices_mut(&mut self) -> &mut Vec<u32> {
        self.update_index_buffer = true;

        &mut self.indices
    }

    #[inline]
    pub fn index_buffer(&mut self) -> &mut Buffer {
        if self.update_index_buffer {
            self.update_index_buffer = false;

            self.index_buffer.write(cast_slice(&self.indices));
        }

        &mut self.index_buffer
    }

    #[inline]
    pub fn get_index_buffer_raw(&self) -> Option<&ike_wgpu::Buffer> {
        self.index_buffer.get_raw()
    }

    #[inline]
    pub fn get<T: AsVertexData>(&self, name: impl AsRef<str>) -> Option<&Vec<T>> {
        T::from_data(&self.vertices.get(name.as_ref())?.data)
    }

    #[inline]
    pub fn get_mut<T: AsVertexData>(&mut self, name: impl AsRef<str>) -> Option<&mut Vec<T>> {
        let attr = self.vertices.get_mut(name.as_ref())?;

        attr.update = true;

        T::from_data_mut(&mut attr.data)
    }

    #[inline]
    pub fn buffer(&mut self, name: impl AsRef<str>) -> Option<&mut Buffer> {
        let attr = self.vertices.get_mut(name.as_ref())?;

        if attr.update {
            attr.update = false;

            attr.buffer.write(attr.data.data());
        }

        Some(&mut attr.buffer)
    }

    #[inline]
    pub fn get_raw_buffer(&self, name: impl AsRef<str>) -> Option<&ike_wgpu::Buffer> {
        let attr = self.vertices.get(name.as_ref())?;

        attr.buffer.get_raw()
    }
}

pub trait IntoVertexData: Sized {
    fn into_data(this: Vec<Self>) -> VertexData;
}

impl IntoVertexData for f32 {
    #[inline]
    fn into_data(this: Vec<f32>) -> VertexData {
        VertexData::Float32(cast_vec(this))
    }
}

impl IntoVertexData for Vec2 {
    #[inline]
    fn into_data(this: Vec<Vec2>) -> VertexData {
        VertexData::Float32x2(cast_vec(this))
    }
}

impl IntoVertexData for Vec3 {
    #[inline]
    fn into_data(this: Vec<Vec3>) -> VertexData {
        VertexData::Float32x3(cast_vec(this))
    }
}

impl IntoVertexData for Vec4 {
    #[inline]
    fn into_data(this: Vec<Vec4>) -> VertexData {
        VertexData::Float32x4(cast_vec(this))
    }
}

impl IntoVertexData for Color {
    #[inline]
    fn into_data(this: Vec<Color>) -> VertexData {
        VertexData::Float32x4(cast_vec(this))
    }
}

impl IntoVertexData for [f32; 2] {
    #[inline]
    fn into_data(this: Vec<[f32; 2]>) -> VertexData {
        VertexData::Float32x3(cast_vec(this))
    }
}

impl IntoVertexData for [f32; 3] {
    #[inline]
    fn into_data(this: Vec<[f32; 3]>) -> VertexData {
        VertexData::Float32x3(cast_vec(this))
    }
}

impl IntoVertexData for [f32; 4] {
    #[inline]
    fn into_data(this: Vec<[f32; 4]>) -> VertexData {
        VertexData::Float32x4(cast_vec(this))
    }
}

pub trait AsVertexData: Sized {
    fn from_data(data: &VertexData) -> Option<&Vec<Self>>;
    fn from_data_mut(data: &mut VertexData) -> Option<&mut Vec<Self>>;
}

impl AsVertexData for f32 {
    fn from_data(data: &VertexData) -> Option<&Vec<Self>> {
        match data {
            VertexData::Float32(data) => Some(data),
            _ => None,
        }
    }

    fn from_data_mut(data: &mut VertexData) -> Option<&mut Vec<Self>> {
        match data {
            VertexData::Float32(data) => Some(data),
            _ => None,
        }
    }
}

impl AsVertexData for [f32; 2] {
    fn from_data(data: &VertexData) -> Option<&Vec<Self>> {
        match data {
            VertexData::Float32x2(data) => Some(data),
            _ => None,
        }
    }

    fn from_data_mut(data: &mut VertexData) -> Option<&mut Vec<Self>> {
        match data {
            VertexData::Float32x2(data) => Some(data),
            _ => None,
        }
    }
}

impl AsVertexData for [f32; 3] {
    fn from_data(data: &VertexData) -> Option<&Vec<Self>> {
        match data {
            VertexData::Float32x3(data) => Some(data),
            _ => None,
        }
    }

    fn from_data_mut(data: &mut VertexData) -> Option<&mut Vec<Self>> {
        match data {
            VertexData::Float32x3(data) => Some(data),
            _ => None,
        }
    }
}

impl AsVertexData for [f32; 4] {
    fn from_data(data: &VertexData) -> Option<&Vec<Self>> {
        match data {
            VertexData::Float32x4(data) => Some(data),
            _ => None,
        }
    }

    fn from_data_mut(data: &mut VertexData) -> Option<&mut Vec<Self>> {
        match data {
            VertexData::Float32x4(data) => Some(data),
            _ => None,
        }
    }
}
