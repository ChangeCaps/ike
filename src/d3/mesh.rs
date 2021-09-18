use std::{
    borrow::Cow,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use bytemuck::{cast_slice, Pod};
use glam::{Vec2, Vec3};

use crate::{
    id::{HasId, Id},
    prelude::Color,
};

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub tangent: Vec3,
    pub bitangent: Vec3,
    pub uv: Vec2,
    pub color: Color,
    pub joints: [u32; 4],
    pub weights: [f32; 4],
}

impl Default for Vertex {
    #[inline]
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            normal: Vec3::Z,
            tangent: Vec3::Y,
            bitangent: Vec3::X,
            uv: Vec2::ZERO,
            color: Color::WHITE,
            joints: [0; 4],
            weights: [0.0; 4],
        }
    }
}

pub trait PositionNormal {
    fn position(&mut self) -> &mut Vec3;
    fn normal(&mut self) -> &mut Vec3;
}

impl PositionNormal for Vertex {
    #[inline]
    fn position(&mut self) -> &mut Vec3 {
        &mut self.position
    }

    #[inline]
    fn normal(&mut self) -> &mut Vec3 {
        &mut self.normal
    }
}

#[derive(Clone, Debug)]
pub struct MeshData<'a> {
    pub vertex_data: Cow<'a, [u8]>,
    pub vertex_count: u32,
    pub index_data: Cow<'a, [u8]>,
    pub index_count: u32,
}

#[derive(Debug)]
pub struct Vertices;

#[derive(Debug)]
pub struct Indices;

#[derive(Debug)]
pub struct Buffer<T, I = T> {
    id: Id<I>,
    inner: Arc<Vec<T>>,
    version: u64,
}

impl<T, I> HasId<I> for Buffer<T, I> {
    #[inline]
    fn id(&self) -> Id<I> {
        self.id
    }
}

impl<T, I> Clone for Buffer<T, I> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            inner: self.inner.clone(),
            version: 1,
        }
    }
}

impl<T, I> Deref for Buffer<T, I> {
    type Target = Vec<T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

impl<T: Clone, I> DerefMut for Buffer<T, I> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.version += 1;

        if Arc::get_mut(&mut self.inner).is_some() {
            return Arc::get_mut(&mut self.inner).unwrap();
        }

        self.id = Id::new();

        Arc::make_mut(&mut self.inner)
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BufferVersion(u64);

impl<T, I> Buffer<T, I> {
    #[inline]
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inner: Default::default(),
            version: 1,
        }
    }

    #[inline]
    pub fn version(&self) -> BufferVersion {
        BufferVersion(self.version)
    }

    #[inline]
    pub fn changed(&self, version: BufferVersion) -> bool {
        self.version != version.0
    }
}

#[derive(Clone, Debug)]
pub struct Mesh<V = Vertex> {
    pub vertices: Buffer<V, Vertices>,
    pub indices: Buffer<u32, Indices>,
    pub pipeline: Option<Id<ike_wgpu::RenderPipeline>>,
}

impl<V> Default for Mesh<V> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<V> HasId<Vertices> for Mesh<V> {
    #[inline]
    fn id(&self) -> Id<Vertices> {
        self.vertices.id()
    }
}

impl<V> HasId<Indices> for Mesh<V> {
    #[inline]
    fn id(&self) -> Id<Indices> {
        self.indices.id()
    }
}

impl<V> Mesh<V> {
    #[inline]
    pub fn new() -> Self {
        Self {
            vertices: Buffer::new(),
            indices: Buffer::new(),
            pipeline: None,
        }
    }

    #[inline]
    pub fn data(&self) -> MeshData<'_>
    where
        V: Pod,
    {
        MeshData {
            vertex_data: cast_slice(&self.vertices).into(),
            vertex_count: self.vertices.len() as u32,
            index_data: cast_slice(&self.indices).into(),
            index_count: self.indices.len() as u32,
        }
    }

    #[inline]
    pub fn calculate_normals(&mut self)
    where
        V: PositionNormal + Clone,
    {
        let vertices = &mut *self.vertices;

        for vertex in vertices.iter_mut() {
            *vertex.normal() = Vec3::ZERO;
        }

        for i in 0..self.indices.len() / 3 {
            let i0 = self.indices[i * 3 + 0];
            let i1 = self.indices[i * 3 + 1];
            let i2 = self.indices[i * 3 + 2];

            let p0 = *vertices[i0 as usize].position();
            let p1 = *vertices[i1 as usize].position();
            let p2 = *vertices[i2 as usize].position();

            let normal = (p1 - p0).cross(p2 - p0);

            *vertices[i0 as usize].normal() += normal;
            *vertices[i1 as usize].normal() += normal;
            *vertices[i2 as usize].normal() += normal;
        }

        for vertex in vertices {
            *vertex.normal() = vertex.normal().normalize();
        }
    }
}

impl Mesh<Vertex> {
    #[inline]
    pub fn calculate_tangents(&mut self) {
        for vertex in self.vertices.iter_mut() {
            vertex.tangent = Vec3::ZERO;
            vertex.bitangent = Vec3::ZERO;
        }

        for i in 0..self.indices.len() / 3 {
            let i0 = self.indices[i * 3 + 0];
            let i1 = self.indices[i * 3 + 1];
            let i2 = self.indices[i * 3 + 2];

            let v0 = &self.vertices[i0 as usize];
            let v1 = &self.vertices[i1 as usize];
            let v2 = &self.vertices[i2 as usize];

            let dp1 = v1.position - v0.position;
            let dp2 = v2.position - v0.position;

            let duv1 = v1.uv - v0.uv;
            let duv2 = v2.uv - v0.uv;

            let r = 1.0 / (duv1.x * duv2.y - duv1.y * duv2.x);
            self.vertices[i0 as usize].tangent += (dp1 * duv2.y - dp2 * duv1.y) * r;
            self.vertices[i0 as usize].bitangent += (dp2 * duv1.x - dp1 * duv2.x) * r;

            self.vertices[i1 as usize].tangent += (dp1 * duv2.y - dp2 * duv1.y) * r;
            self.vertices[i1 as usize].bitangent += (dp2 * duv1.x - dp1 * duv2.x) * r;

            self.vertices[i2 as usize].tangent += (dp1 * duv2.y - dp2 * duv1.y) * r;
            self.vertices[i2 as usize].bitangent += (dp2 * duv1.x - dp1 * duv2.x) * r;
        }

        for vertex in self.vertices.iter_mut() {
            vertex.tangent = vertex.tangent.normalize();
            vertex.bitangent = vertex.bitangent.normalize();
        }
    }
}
