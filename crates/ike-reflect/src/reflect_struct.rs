use crate::{Reflect, ReflectMut, ReflectRef};

pub trait ReflectStruct: Reflect {
    fn field(&self, name: &str) -> Option<&dyn Reflect>;
    fn field_mut(&mut self, name: &str) -> Option<&mut dyn Reflect>;
    fn field_at(&self, index: usize) -> Option<&dyn Reflect>;
    fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn Reflect>;
    fn name_at(&self, index: usize) -> Option<&str>;
    fn field_len(&self) -> usize;

    fn partial_eq(&self, other: &dyn ReflectStruct) -> bool {
        if self.type_name() != other.type_name() {
            return false;
        }

        for i in 0..self.field_len() {
            if let (Some(a), Some(b)) = (self.field_at(i), other.field_at(i)) {
                if !a.partial_eq(b) {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }
}

struct DynamicField {
    name: String,
    value: Box<dyn Reflect>,
}

#[derive(Default)]
pub struct DynamicStruct {
    name: String,
    fields: Vec<DynamicField>,
}

impl DynamicStruct {
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    pub fn push_boxed(&mut self, name: impl Into<String>, value: Box<dyn Reflect>) {
        self.fields.push(DynamicField {
            name: name.into(),
            value,
        });
    }
}

impl ReflectStruct for DynamicStruct {
    fn field(&self, name: &str) -> Option<&dyn Reflect> {
        self.fields.iter().find_map(|field| {
            if field.name == name {
                Some(field.value.as_ref())
            } else {
                None
            }
        })
    }

    fn field_mut(&mut self, name: &str) -> Option<&mut dyn Reflect> {
        self.fields.iter_mut().find_map(|field| {
            if field.name == name {
                Some(field.value.as_mut())
            } else {
                None
            }
        })
    }

    fn field_at(&self, index: usize) -> Option<&dyn Reflect> {
        self.fields.get(index).map(|field| field.value.as_ref())
    }

    fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
        self.fields.get_mut(index).map(|field| field.value.as_mut())
    }

    fn name_at(&self, index: usize) -> Option<&str> {
        self.fields.get(index).map(|field| field.name.as_str())
    }

    fn field_len(&self) -> usize {
        self.fields.len()
    }
}

impl Reflect for DynamicStruct {
    fn type_name(&self) -> &str {
        &self.name
    }

    fn reflect_ref(&self) -> ReflectRef {
        ReflectRef::Struct(self)
    }

    fn reflect_mut(&mut self) -> ReflectMut {
        ReflectMut::Struct(self)
    }
}
