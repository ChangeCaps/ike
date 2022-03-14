use crate::{Reflect, ReflectStruct, ReflectTuple};

pub trait ReflectEnum: Reflect {
    fn variant_name(&self) -> &str;
    fn variant_ref(&self) -> VariantRef;
    fn variant_mut(&mut self) -> VariantMut;
}

pub enum VariantRef<'a> {
    Tuple(&'a dyn ReflectTuple),
    Struct(&'a dyn ReflectStruct),
}

pub enum VariantMut<'a> {
    Tuple(&'a mut dyn ReflectTuple),
    Struct(&'a mut dyn ReflectStruct),
}

impl<'a> VariantRef<'a> {
    pub fn get_tuple(&self) -> Option<&'a dyn ReflectTuple> {
        match *self {
            Self::Tuple(reflect) => Some(reflect),
            _ => None,
        }
    }

    pub fn get_struct(&self) -> Option<&'a dyn ReflectStruct> {
        match *self {
            Self::Struct(reflect) => Some(reflect),
            _ => None,
        }
    }
}

impl<'a> VariantMut<'a> {
    pub fn get_tuple(&self) -> Option<&dyn ReflectTuple> {
        match self {
            Self::Tuple(reflect) => Some(*reflect),
            _ => None,
        }
    }

    pub fn get_struct(&self) -> Option<&dyn ReflectStruct> {
        match self {
            Self::Struct(reflect) => Some(*reflect),
            _ => None,
        }
    }

    pub fn get_tuple_mut(&mut self) -> Option<&mut dyn ReflectTuple> {
        match self {
            Self::Tuple(reflect) => Some(*reflect),
            _ => None,
        }
    }

    pub fn get_struct_mut(&mut self) -> Option<&mut dyn ReflectStruct> {
        match self {
            Self::Struct(reflect) => Some(*reflect),
            _ => None,
        }
    }
}
