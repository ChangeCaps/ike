use std::any::Any;

use crate::{Reflect, ReflectMut, ReflectRef};

pub trait TupleStruct: Reflect {
    fn field(&self, index: usize) -> Option<&dyn Reflect>;
    fn field_mut(&mut self, index: usize) -> Option<&mut dyn Reflect>;
    fn field_len(&self) -> usize;
    fn clone_dynamic(&self) -> DynamicTupleStruct;
}

#[derive(Default)]
pub struct DynamicTupleStruct {
    name: String,
    fields: Vec<Box<dyn Reflect>>,
}

impl DynamicTupleStruct {
    #[inline]
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    #[inline]
    pub fn push_boxed(&mut self, value: Box<dyn Reflect>) {
        self.fields.push(value);
    }
}

impl TupleStruct for DynamicTupleStruct {
    #[inline]
    fn field(&self, index: usize) -> Option<&dyn Reflect> {
        self.fields.get(index).map(|field| field.as_ref())
    }

    #[inline]
    fn field_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
        self.fields.get_mut(index).map(|field| field.as_mut())
    }

    #[inline]
    fn field_len(&self) -> usize {
        self.fields.len()
    }

    #[inline]
    fn clone_dynamic(&self) -> DynamicTupleStruct {
        Self {
            name: self.name.clone(),
            fields: self
                .fields
                .iter()
                .map(|field| field.clone_value())
                .collect(),
        }
    }
}

unsafe impl Reflect for DynamicTupleStruct {
    #[inline]
    fn type_name(&self) -> &str {
        &self.name
    }

    #[inline]
    fn any(&self) -> &dyn Any {
        self
    }

    #[inline]
    fn any_mut(&mut self) -> &mut dyn Any {
        self
    }

    #[inline]
    fn reflect_ref(&self) -> ReflectRef {
        ReflectRef::TupleStruct(self)
    }

    #[inline]
    fn reflect_mut(&mut self) -> ReflectMut {
        ReflectMut::TupleStruct(self)
    }

    #[inline]
    fn clone_value(&self) -> Box<dyn Reflect> {
        Box::new(self.clone_dynamic())
    }

    #[inline]
    fn partial_eq(&self, other: &dyn Reflect) -> bool {
        match other.reflect_ref() {
            ReflectRef::TupleStruct(value) => {
                if self.field_len() == value.field_len() {
                    for i in 0..self.field_len() {
                        if self.field(i).unwrap().partial_eq(value.field(i).unwrap()) {
                            return false;
                        }
                    }

                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    #[inline]
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self>
    where
        Self: Sized,
    {
        match reflect.reflect_ref() {
            ReflectRef::TupleStruct(value) => Some(value.clone_dynamic()),
            _ => None,
        }
    }

    #[inline]
    fn default_value() -> Self
    where
        Self: Sized,
    {
        Default::default()
    }
}
