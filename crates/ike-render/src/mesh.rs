use bytemuck::{cast_slice, cast_slice_mut, cast_vec};
use ike_math::{Vec2, Vec3, Vec4};
use std::borrow::Cow;

use crate::{Buffer, BufferInitDescriptor, BufferUsages, Color, RenderDevice};

#[derive(Clone, Debug)]
pub enum VertexAttribute {
    Float32(Vec<f32>),
    Float32x2(Vec<[f32; 2]>),
    Float32x3(Vec<[f32; 3]>),
    Float32x4(Vec<[f32; 4]>),
}

impl VertexAttribute {
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Float32(values) => cast_slice(values),
            Self::Float32x2(values) => cast_slice(values),
            Self::Float32x3(values) => cast_slice(values),
            Self::Float32x4(values) => cast_slice(values),
        }
    }
}

pub trait AsVertexAttribute: Sized {
    fn into(vec: Vec<Self>) -> VertexAttribute;

    fn from(attribute: VertexAttribute) -> Option<Vec<Self>>;

    fn as_ref(attribute: &VertexAttribute) -> Option<&[Self]>;

    fn as_mut(attribute: &mut VertexAttribute) -> Option<&mut [Self]>;
}

macro_rules! impl_as_vertex_attribute {
    ($ty:ty, $attr:ident) => {
        impl AsVertexAttribute for $ty {
            fn into(vec: Vec<Self>) -> VertexAttribute {
                VertexAttribute::$attr(cast_vec(vec))
            }

            fn from(attribute: VertexAttribute) -> Option<Vec<Self>> {
                match attribute {
                    VertexAttribute::$attr(vec) => Some(cast_vec(vec)),
                    _ => None,
                }
            }

            fn as_ref(attribute: &VertexAttribute) -> Option<&[Self]> {
                match attribute {
                    VertexAttribute::$attr(vec) => Some(cast_slice(vec)),
                    _ => None,
                }
            }

            fn as_mut(attribute: &mut VertexAttribute) -> Option<&mut [Self]> {
                match attribute {
                    VertexAttribute::$attr(vec) => Some(cast_slice_mut(vec)),
                    _ => None,
                }
            }
        }
    };
}

impl_as_vertex_attribute!(f32, Float32);
impl_as_vertex_attribute!([f32; 2], Float32x2);
impl_as_vertex_attribute!([f32; 3], Float32x3);
impl_as_vertex_attribute!([f32; 4], Float32x4);

impl_as_vertex_attribute!(Vec2, Float32x2);
impl_as_vertex_attribute!(Vec3, Float32x3);
impl_as_vertex_attribute!(Vec4, Float32x4);

impl_as_vertex_attribute!(Color, Float32x4);

#[derive(Clone, Debug, Default)]
pub struct Mesh {
    attributes: Vec<(Cow<'static, str>, VertexAttribute)>,
    indices: Vec<u32>,
}

impl Mesh {
    pub const POSITION: &'static str = "position";
    pub const NORMAL: &'static str = "normal";
    pub const TANGENT: &'static str = "tangent";
    pub const UV_0: &'static str = "uv_0";
    pub const COLOR_0: &'static str = "color_0";

    pub const fn new() -> Self {
        Self {
            attributes: Vec::new(),
            indices: Vec::new(),
        }
    }

    fn get_index(&self, name: impl Into<Cow<'static, str>>) -> Option<usize> {
        let name = name.into();

        self.attributes
            .iter()
            .position(|(attr_name, _)| *attr_name == name)
    }

    pub fn insert_attribute<T: AsVertexAttribute>(
        &mut self,
        name: impl Into<Cow<'static, str>>,
        attribute: Vec<T>,
    ) {
        let name = name.into();

        if let Some(index) = self.get_index(name.clone()) {
            self.attributes[index].1 = T::into(attribute);
        } else {
            self.attributes.push((name, T::into(attribute)));
        }
    }

    pub fn remove_attribute<T: AsVertexAttribute>(
        &mut self,
        name: impl Into<Cow<'static, str>>,
    ) -> Option<Vec<T>> {
        let index = self.get_index(name)?;
        let attribute = self.attributes.remove(index).1;

        T::from(attribute)
    }

    pub fn get_attribute<T: AsVertexAttribute>(
        &self,
        name: impl Into<Cow<'static, str>>,
    ) -> Option<&[T]> {
        let index = self.get_index(name)?;
        let attribute = &self.attributes[index].1;

        T::as_ref(attribute)
    }

    pub fn get_attribute_mut<T: AsVertexAttribute>(
        &mut self,
        name: impl Into<Cow<'static, str>>,
    ) -> Option<&mut [T]> {
        let index = self.get_index(name)?;
        let attribute = &mut self.attributes[index].1;

        T::as_mut(attribute)
    }

    pub fn get_indices(&self) -> &Vec<u32> {
        &self.indices
    }

    pub fn get_indices_mut(&mut self) -> &mut Vec<u32> {
        &mut self.indices
    }
}

pub struct MeshBuffers {
    attributes: Vec<(Cow<'static, str>, Buffer)>,
    indices: Buffer,
}

impl MeshBuffers {
    pub fn from_mesh(mesh: &Mesh, device: &RenderDevice) -> Self {
        let attributes = mesh
            .attributes
            .iter()
            .map(|(name, attribute)| {
                let label = format!("ike_mesh_attribute({})_buffer", name);

                let buffer = device.create_buffer_init(&BufferInitDescriptor {
                    label: Some(&label),
                    contents: attribute.as_bytes(),
                    usage: BufferUsages::COPY_SRC | BufferUsages::VERTEX,
                });

                (name.clone(), buffer)
            })
            .collect();

        let indices = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("ike_mesh_index_buffer"),
            contents: cast_slice(&mesh.indices),
            usage: BufferUsages::COPY_SRC | BufferUsages::INDEX,
        });

        Self {
            attributes,
            indices,
        }
    }

    fn get_index(&self, name: impl AsRef<str>) -> Option<usize> {
        self.attributes
            .iter()
            .position(|(attr_name, _)| *attr_name == name.as_ref())
    }

    pub fn get_attribute(&self, name: impl AsRef<str>) -> Option<&Buffer> {
        let index = self.get_index(name)?;
        Some(&self.attributes[index].1)
    }

    pub fn get_indices(&self) -> &Buffer {
        &self.indices
    }
}
