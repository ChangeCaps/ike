use std::any::{type_name, Any, TypeId};

use crate::{
    ReflectEnum, ReflectList, ReflectMap, ReflectSet, ReflectStruct, ReflectTuple, ReflectValue,
};

pub enum ReflectRef<'a> {
    Tuple(&'a dyn ReflectTuple),
    Struct(&'a dyn ReflectStruct),
    Enum(&'a dyn ReflectEnum),
    List(&'a dyn ReflectList),
    Set(&'a dyn ReflectSet),
    Map(&'a dyn ReflectMap),
    Value(&'a dyn ReflectValue),
}

pub enum ReflectMut<'a> {
    Tuple(&'a mut dyn ReflectTuple),
    Struct(&'a mut dyn ReflectStruct),
    Enum(&'a mut dyn ReflectEnum),
    List(&'a mut dyn ReflectList),
    Set(&'a mut dyn ReflectSet),
    Map(&'a mut dyn ReflectMap),
    Value(&'a mut dyn ReflectValue),
}

pub trait Reflect: FromReflect + Any {
    fn type_name(&self) -> &str {
        type_name::<Self>()
    }

    fn reflect_ref(&self) -> ReflectRef;
    fn reflect_mut(&mut self) -> ReflectMut;
}

pub trait FromReflect {
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self>
    where
        Self: Sized;
}

impl dyn Reflect {
    pub fn downcast<T: Reflect>(self: Box<Self>) -> Result<Box<T>, Box<Self>> {
        if Any::type_id(self.as_ref()) == TypeId::of::<T>() {
            // SAFETY:
            // we just checked that self has the same TypeId as T
            // so since types are the same casting is safe
            unsafe { Ok(Box::from_raw(Box::into_raw(self) as *mut T)) }
        } else {
            Err(self)
        }
    }

    pub fn is<T: Reflect>(&self) -> bool {
        Any::type_id(self) == TypeId::of::<T>()
    }

    pub fn downcast_ref<T: Reflect>(&self) -> Option<&T> {
        if Any::type_id(self) == TypeId::of::<T>() {
            // SAFETY:
            // we just checked that self has the same TypeId as T
            // so since types are the same casting is safe
            unsafe { Some(&*(self as *const _ as *const T)) }
        } else {
            None
        }
    }

    pub fn downcast_mut<T: Reflect>(&mut self) -> Option<&mut T> {
        if Any::type_id(self) == TypeId::of::<T>() {
            // SAFETY:
            // we just checked that self has the same TypeId as T
            // so since types are the same casting is safe
            unsafe { Some(&mut *(self as *mut _ as *mut T)) }
        } else {
            None
        }
    }
}

impl<'a> ReflectRef<'a> {
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

    pub fn get_enum(&self) -> Option<&'a dyn ReflectEnum> {
        match *self {
            Self::Enum(reflect) => Some(reflect),
            _ => None,
        }
    }

    pub fn get_list(&self) -> Option<&'a dyn ReflectList> {
        match *self {
            Self::List(reflect) => Some(reflect),
            _ => None,
        }
    }

    pub fn get_set(&self) -> Option<&'a dyn ReflectSet> {
        match *self {
            Self::Set(reflect) => Some(reflect),
            _ => None,
        }
    }

    pub fn get_map(&self) -> Option<&'a dyn ReflectMap> {
        match *self {
            Self::Map(reflect) => Some(reflect),
            _ => None,
        }
    }

    pub fn get_value(&self) -> Option<&'a dyn ReflectValue> {
        match *self {
            Self::Value(reflect) => Some(reflect),
            _ => None,
        }
    }
}

impl<'a> ReflectMut<'a> {
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

    pub fn get_enum(&self) -> Option<&dyn ReflectEnum> {
        match self {
            Self::Enum(reflect) => Some(*reflect),
            _ => None,
        }
    }

    pub fn get_list(&self) -> Option<&dyn ReflectList> {
        match self {
            Self::List(reflect) => Some(*reflect),
            _ => None,
        }
    }

    pub fn get_set(&self) -> Option<&dyn ReflectSet> {
        match self {
            Self::Set(reflect) => Some(*reflect),
            _ => None,
        }
    }

    pub fn get_map(&self) -> Option<&dyn ReflectMap> {
        match self {
            Self::Map(reflect) => Some(*reflect),
            _ => None,
        }
    }

    pub fn get_value(&self) -> Option<&dyn ReflectValue> {
        match self {
            Self::Value(reflect) => Some(*reflect),
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

    pub fn get_enum_mut(&mut self) -> Option<&mut dyn ReflectEnum> {
        match self {
            Self::Enum(reflect) => Some(*reflect),
            _ => None,
        }
    }

    pub fn get_list_mut(&mut self) -> Option<&mut dyn ReflectList> {
        match self {
            Self::List(reflect) => Some(*reflect),
            _ => None,
        }
    }

    pub fn get_set_mut(&mut self) -> Option<&mut dyn ReflectSet> {
        match self {
            Self::Set(reflect) => Some(*reflect),
            _ => None,
        }
    }

    pub fn get_map_mut(&mut self) -> Option<&mut dyn ReflectMap> {
        match self {
            Self::Map(reflect) => Some(*reflect),
            _ => None,
        }
    }

    pub fn get_value_mut(&mut self) -> Option<&mut dyn ReflectValue> {
        match self {
            Self::Value(reflect) => Some(*reflect),
            _ => None,
        }
    }
}

impl<T: Reflect> Reflect for Box<T> {
    fn type_name(&self) -> &str {
        self.as_ref().type_name()
    }

    fn reflect_ref(&self) -> ReflectRef {
        self.as_ref().reflect_ref()
    }

    fn reflect_mut(&mut self) -> ReflectMut {
        self.as_mut().reflect_mut()
    }
}

impl<T: FromReflect> FromReflect for Box<T> {
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        Some(Box::new(T::from_reflect(reflect)?))
    }
}
