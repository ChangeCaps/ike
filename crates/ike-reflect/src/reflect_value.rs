use std::{
    any::{Any, TypeId},
    borrow::Cow,
};

#[cfg(feature = "serialize")]
use erased_serde::Serialize;
use ike_math::{
    IVec2, IVec3, IVec4, Mat2, Mat3, Mat3A, Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec3A,
    Vec4,
};

use crate::{FromReflect, Reflect, ReflectMut, ReflectRef};

pub trait ReflectValue: Reflect {
    #[cfg(feature = "serialize")]
    fn serialize(&self) -> &dyn Serialize;
    fn partial_eq(&self, other: &dyn ReflectValue) -> bool;
    fn value_clone(&self) -> Box<dyn Reflect>;
}

impl dyn ReflectValue {
    pub fn downcast<T: ReflectValue>(self: Box<Self>) -> Result<Box<T>, Box<Self>> {
        if Any::type_id(self.as_ref()) == TypeId::of::<T>() {
            // SAFETY:
            // we just checked that self has the same TypeId as T
            // so since types are the same casting is safe
            unsafe { Ok(Box::from_raw(Box::into_raw(self) as *mut T)) }
        } else {
            Err(self)
        }
    }

    pub fn downcast_ref<T: ReflectValue>(&self) -> Option<&T> {
        if Any::type_id(self) == TypeId::of::<T>() {
            // SAFETY:
            // we just checked that self has the same TypeId as T
            // so since types are the same casting is safe
            unsafe { Some(&*(self as *const _ as *const T)) }
        } else {
            None
        }
    }

    pub fn downcast_mut<T: ReflectValue>(&mut self) -> Option<&mut T> {
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

macro_rules! impl_reflect_value {
    ($type:ty) => {
        impl Reflect for $type {
            fn reflect_ref(&self) -> ReflectRef {
                ReflectRef::Value(self)
            }

            fn reflect_mut(&mut self) -> ReflectMut {
                ReflectMut::Value(self)
            }
        }

        impl ReflectValue for $type {
            #[cfg(feature = "serialize")]
            fn serialize(&self) -> &dyn Serialize {
                self
            }

            fn partial_eq(&self, other: &dyn ReflectValue) -> bool {
                if let Some(other) = other.downcast_ref::<Self>() {
                    self == other
                } else {
                    false
                }
            }

            fn value_clone(&self) -> Box<dyn Reflect> {
                Box::new(self.clone())
            }
        }

        impl FromReflect for $type {
            fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
                reflect.downcast_ref::<Self>().cloned()
            }
        }
    };
}

impl_reflect_value!(u8);
impl_reflect_value!(u16);
impl_reflect_value!(u32);
impl_reflect_value!(u64);
impl_reflect_value!(u128);
impl_reflect_value!(usize);

impl_reflect_value!(i8);
impl_reflect_value!(i16);
impl_reflect_value!(i32);
impl_reflect_value!(i64);
impl_reflect_value!(i128);
impl_reflect_value!(isize);

impl_reflect_value!(f32);
impl_reflect_value!(f64);

impl_reflect_value!(bool);

impl_reflect_value!(Cow<'static, str>);
impl_reflect_value!(String);

impl_reflect_value!(UVec2);
impl_reflect_value!(UVec3);
impl_reflect_value!(UVec4);

impl_reflect_value!(IVec2);
impl_reflect_value!(IVec3);
impl_reflect_value!(IVec4);

impl_reflect_value!(Vec2);
impl_reflect_value!(Vec3);
impl_reflect_value!(Vec3A);
impl_reflect_value!(Vec4);

impl_reflect_value!(Quat);

impl_reflect_value!(Mat2);
impl_reflect_value!(Mat3);
impl_reflect_value!(Mat3A);
impl_reflect_value!(Mat4);
