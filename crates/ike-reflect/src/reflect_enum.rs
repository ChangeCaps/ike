use std::any::type_name;

use crate::{FromReflect, Reflect, ReflectMut, ReflectRef, ReflectStruct, ReflectTuple};

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

impl<T: FromReflect> FromReflect for Option<T> {
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        if reflect.type_name() == type_name::<Self>() {
            let reflect = reflect.reflect_ref().get_enum()?;

            match reflect.variant_name() {
                "Some" => {
                    let reflect = reflect.variant_ref().get_tuple()?;

                    Some(T::from_reflect(reflect.field(0)?))
                }
                "None" => Some(None),
                _ => None,
            }
        } else {
            None
        }
    }
}

impl<T: Reflect> ReflectStruct for Option<T> {
    fn field(&self, _name: &str) -> Option<&dyn Reflect> {
        None
    }

    fn field_mut(&mut self, _name: &str) -> Option<&mut dyn Reflect> {
        None
    }

    fn field_at(&self, _index: usize) -> Option<&dyn Reflect> {
        None
    }

    fn field_at_mut(&mut self, _index: usize) -> Option<&mut dyn Reflect> {
        None
    }

    fn name_at(&self, _index: usize) -> Option<&str> {
        None
    }

    fn field_len(&self) -> usize {
        0
    }
}

impl<T: Reflect> ReflectTuple for Option<T> {
    fn field(&self, index: usize) -> Option<&dyn Reflect> {
        if index != 0 {
            return None;
        }

        match self {
            Some(reflect) => Some(reflect),
            None => None,
        }
    }

    fn field_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
        if index != 0 {
            return None;
        }

        match self {
            Some(reflect) => Some(reflect),
            None => None,
        }
    }

    fn field_len(&self) -> usize {
        match self {
            Some(_) => 1,
            None => 0,
        }
    }
}

impl<T: Reflect> ReflectEnum for Option<T> {
    fn variant_name(&self) -> &str {
        match self {
            Some(_) => "Some",
            None => "Non",
        }
    }

    fn variant_ref(&self) -> VariantRef {
        match self {
            Some(_) => VariantRef::Tuple(self),
            None => VariantRef::Struct(self),
        }
    }

    fn variant_mut(&mut self) -> VariantMut {
        match self {
            Some(_) => VariantMut::Tuple(self),
            None => VariantMut::Struct(self),
        }
    }
}

impl<T: Reflect> Reflect for Option<T> {
    fn reflect_ref(&self) -> ReflectRef {
        ReflectRef::Enum(self)
    }

    fn reflect_mut(&mut self) -> ReflectMut {
        ReflectMut::Enum(self)
    }
}
