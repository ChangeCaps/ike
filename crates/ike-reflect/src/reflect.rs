use std::any::{Any, TypeId};

use crate::{DynamicMap, Enum, Map, Struct, TupleStruct, Value};

pub enum ReflectRef<'a> {
    Struct(&'a dyn Struct),
    TupleStruct(&'a dyn TupleStruct),
    Enum(&'a dyn Enum),
    Map(&'a dyn Map),
    Value(&'a dyn Value),
}

pub enum ReflectMut<'a> {
    Struct(&'a mut dyn Struct),
    TupleStruct(&'a mut dyn TupleStruct),
    Enum(&'a mut dyn Enum),
    Map(&'a mut dyn Map),
    Value(&'a mut dyn Value),
}

pub unsafe trait Reflect: Send + Sync + 'static {
    fn type_name(&self) -> &str;
    fn any(&self) -> &dyn Any;
    fn any_mut(&mut self) -> &mut dyn Any;
    fn reflect_ref(&self) -> ReflectRef;
    fn reflect_mut(&mut self) -> ReflectMut;
    fn clone_value(&self) -> Box<dyn Reflect>;
}

pub trait FromReflect: Reflect + Sized {
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self>;
}

impl dyn Reflect {
    #[inline]
    pub fn downcast<T: Reflect>(self: Box<dyn Reflect>) -> Result<Box<T>, Box<dyn Reflect>> {
        if self.as_ref().any().type_id() == TypeId::of::<T>() {
            let raw: *mut dyn Reflect = Box::into_raw(self);

            unsafe { Ok(Box::from_raw(raw as *mut T)) }
        } else {
            Err(self)
        }
    }

    #[inline]
    pub fn downcast_mut<T: Reflect>(&mut self) -> Option<&mut T> {
        self.any_mut().downcast_mut()
    }

    #[inline]
    pub fn reflect_into<T: FromReflect>(&self) -> Option<T> {
        FromReflect::from_reflect(self)
    }

    #[inline]
    pub fn map_values<T: Reflect>(&self, f: impl FnMut(&mut T)) -> Box<dyn Reflect> {
        let mut reflect = self.clone_value();

        reflect
    }
}

#[inline]
fn map_values<T: Reflect>(reflect: &mut dyn Reflect, f: &mut impl FnMut(&mut T)) {}

#[inline]
fn map_values_struct<T: Reflect>(reflect: &mut dyn Struct, f: &mut impl FnMut(&mut T)) {
    for i in 0..reflect.field_len() {
        map_values(reflect.field_at_mut(i).unwrap(), f);
    }
}

#[inline]
fn map_values_tuple_struct<T: Reflect>(reflect: &mut dyn TupleStruct, f: &mut impl FnMut(&mut T)) {
    for i in 0..reflect.field_len() {
        map_values(reflect.field_mut(i).unwrap(), f);
    }
}

#[inline]
fn map_values_map<T: Reflect>(reflect: &mut DynamicMap, f: &mut impl FnMut(&mut T)) {}
