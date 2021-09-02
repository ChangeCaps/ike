use std::{borrow::Cow, ops::{Deref, DerefMut}, sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    }};

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
    pub uv: Vec2,
    pub color: Color,
}

pub trait PositionNormal {
    fn position(&self) -> Vec3;
    fn position_mut(&mut self) -> &mut Vec3;
    fn normal(&self) -> Vec3;
    fn normal_mut(&mut self) -> &mut Vec3;
}

impl PositionNormal for Vertex {
    #[inline]
    fn position(&self) -> Vec3 {
        self.position
    }

    #[inline]
    fn position_mut(&mut self) -> &mut Vec3 {
        &mut self.position
    }

    #[inline]
    fn normal(&self) -> Vec3 {
        self.normal
    }

    #[inline]
    fn normal_mut(&mut self) -> &mut Vec3 {
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
	mutated: AtomicBool,
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
			mutated: AtomicBool::new(true),
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
		self.mutated.store(true, Ordering::SeqCst);

		if Arc::get_mut(&mut self.inner).is_some() {
			return Arc::get_mut(&mut self.inner).unwrap();
		}	

		self.id = Id::new();

		Arc::make_mut(&mut self.inner)
	}	
}

impl<T, I> Buffer<T, I> {
	#[inline]
	pub fn new() -> Self {
		Self {
			id: Id::new(),
			inner: Default::default(),
			mutated: AtomicBool::new(false),
		}
	}

	#[inline]
	pub fn mutated(&self) -> bool {
		self.mutated.load(Ordering::SeqCst)
	}

	#[inline]
	pub fn reset_mutated(&self) {
		self.mutated.store(false, Ordering::SeqCst);
	}
}

#[derive(Debug)]
pub struct Mesh<V = Vertex> {
    pub vertices: Buffer<V, Vertices>,
    pub indices: Buffer<u32, Indices>,
}

impl<V> Default for Mesh<V> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<V> Clone for Mesh<V> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            vertices: self.vertices.clone(),
            indices: self.indices.clone(),
        }
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
        }
    }

	#[inline]
	pub fn mutated(&self) -> bool {
		self.vertices.mutated() | self.indices.mutated()
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
            *vertex.normal_mut() = Vec3::ZERO;
        }

        for i in 0..self.indices.len() / 3 {
            let i0 = self.indices[i * 3 + 0];
            let i1 = self.indices[i * 3 + 1];
            let i2 = self.indices[i * 3 + 2];

            let p0 = vertices[i0 as usize].position();
            let p1 = vertices[i1 as usize].position();
            let p2 = vertices[i2 as usize].position();

            let normal = (p1 - p0).cross(p2 - p0).normalize();

            *vertices[i0 as usize].normal_mut() += normal;
            *vertices[i1 as usize].normal_mut() += normal;
            *vertices[i2 as usize].normal_mut() += normal;
        }

        for vertex in vertices {
            *vertex.normal_mut() = vertex.normal().normalize();
        }
    }	
}