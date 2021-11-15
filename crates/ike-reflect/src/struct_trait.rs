use std::{any::Any, borrow::Cow, collections::HashMap};

use crate::{Reflect, ReflectMut, ReflectRef};

pub trait Struct: Reflect {
    fn field(&self, name: &str) -> Option<&dyn Reflect>;
    fn field_mut(&mut self, name: &str) -> Option<&mut dyn Reflect>;
    fn field_at(&self, index: usize) -> Option<&dyn Reflect>;
    fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn Reflect>;
    fn name_at(&self, index: usize) -> Option<&str>;
    fn field_len(&self) -> usize;
}

pub struct FieldIter<'a> {
    index: usize,
    value: &'a dyn Struct,
}

impl<'a> Iterator for FieldIter<'a> {
    type Item = &'a dyn Reflect;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let value = self.value.field_at(self.index);
        self.index += 1;
        value
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.value.field_len();
        (size, Some(size))
    }
}

impl dyn Struct {
    #[inline]
    pub fn iter_fields(&self) -> FieldIter {
        FieldIter {
            index: 0,
            value: self,
        }
    }
}

#[derive(Default)]
pub struct DynamicStruct {
    name: String,
    fields: Vec<Box<dyn Reflect>>,
    field_names: Vec<Cow<'static, str>>,
    field_indices: HashMap<Cow<'static, str>, usize>,
}

impl DynamicStruct {
    #[inline]
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    #[inline]
    pub fn insert_boxed(&mut self, name: &str, value: Box<dyn Reflect>) {
        if let Some(idx) = self.field_indices.get(name) {
            self.fields[*idx] = value;
        } else {
            let name = Cow::Owned(String::from(name));

            let idx = self.fields.len();

            self.fields.push(value);
            self.field_indices.insert(name, idx);
        }
    }

    #[inline]
    pub fn insert<T: Reflect>(&mut self, name: &str, value: T) {
        self.insert_boxed(name, Box::new(value))
    }
}

impl Struct for DynamicStruct {
    #[inline]
    fn field(&self, name: &str) -> Option<&dyn Reflect> {
        let index = self.field_indices.get(name)?;

        Some(self.fields[*index].as_ref())
    }

    #[inline]
    fn field_mut(&mut self, name: &str) -> Option<&mut dyn Reflect> {
        let index = self.field_indices.get(name)?;

        Some(self.fields[*index].as_mut())
    }

    #[inline]
    fn field_at(&self, index: usize) -> Option<&dyn Reflect> {
        self.fields.get(index).map(|field| field.as_ref())
    }

    #[inline]
    fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
        self.fields.get_mut(index).map(|field| field.as_mut())
    }

    #[inline]
    fn name_at(&self, index: usize) -> Option<&str> {
        self.field_names.get(index).map(|name| name.as_ref())
    }

    #[inline]
    fn field_len(&self) -> usize {
        self.field_indices.len()
    }
}

unsafe impl Reflect for DynamicStruct {
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
        ReflectRef::Struct(self)
    }

    #[inline]
    fn reflect_mut(&mut self) -> ReflectMut {
        ReflectMut::Struct(self)
    }

    #[inline]
    fn clone_value(&self) -> Box<dyn Reflect> {
        Box::new(Self {
            name: self.name.clone(),
            fields: self
                .fields
                .iter()
                .map(|field| field.clone_value())
                .collect(),
            field_names: self.field_names.clone(),
            field_indices: self.field_indices.clone(),
        })
    }

    #[inline]
    fn partial_eq(&self, other: &dyn Reflect) -> bool {
        match other.reflect_ref() {
            ReflectRef::Struct(value) => {
                if self.field_len() == value.field_len() {
                    for i in 0..self.field_len() {
                        if self
                            .field_at(i)
                            .unwrap()
                            .partial_eq(value.field_at(i).unwrap())
                        {
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
}
