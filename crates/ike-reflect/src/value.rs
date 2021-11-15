pub use erased_serde::Serialize;
use ike_core::Entity;

use crate::{
    FromReflect, FromType, Reflect, ReflectComponent, ReflectDeserialize, ReflectMut, ReflectRef,
    RegisterType, TypeRegistration, TypeRegistry,
};

use glam::*;
use std::{
    any::{type_name, Any, TypeId},
    path::PathBuf,
};

pub trait Value: Reflect {
    fn serialize(&self) -> &dyn Serialize;
}

macro_rules! impl_reflect_value {
    ($ty:path) => {
        impl RegisterType for $ty {
            #[inline]
            fn register(type_registry: &mut TypeRegistry) {
                if !type_registry.contains(&TypeId::of::<Self>()) {
                    let mut registration = TypeRegistration::from_type::<Self>();

                    registration.insert(<ReflectComponent as FromType<Self>>::from_type());
                    registration.insert(<ReflectDeserialize as FromType<Self>>::from_type());

                    type_registry.insert(registration);
                }
            }
        }

        impl Value for $ty {
            #[inline]
            fn serialize(&self) -> &dyn Serialize {
                self
            }
        }

        impl FromReflect for $ty {
            #[inline]
            fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
                if reflect.any().is::<Self>() {
                    reflect.clone_value().downcast().ok().map(|value| *value)
                } else {
                    None
                }
            }
        }

        unsafe impl Reflect for $ty {
            #[inline]
            fn type_name(&self) -> &str {
                type_name::<Self>()
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
                ReflectRef::Value(self)
            }

            #[inline]
            fn reflect_mut(&mut self) -> ReflectMut {
                ReflectMut::Value(self)
            }

            #[inline]
            fn clone_value(&self) -> Box<dyn Reflect> {
                Box::new(self.clone())
            }

            #[inline]
            fn partial_eq(&self, other: &dyn Reflect) -> bool {
                if let Some(other) = other.downcast_ref::<Self>() {
                    self == other
                } else {
                    false
                }
            }
        }
    };
}

// glam
impl_reflect_value!(Vec2);
impl_reflect_value!(Vec3);
impl_reflect_value!(Vec4);
impl_reflect_value!(IVec2);
impl_reflect_value!(IVec3);
impl_reflect_value!(IVec4);
impl_reflect_value!(UVec2);
impl_reflect_value!(UVec3);
impl_reflect_value!(UVec4);
impl_reflect_value!(Mat2);
impl_reflect_value!(Mat3);
impl_reflect_value!(Mat4);
impl_reflect_value!(Quat);

// std
impl_reflect_value!(i8);
impl_reflect_value!(i16);
impl_reflect_value!(i32);
impl_reflect_value!(i64);
impl_reflect_value!(i128);
impl_reflect_value!(u8);
impl_reflect_value!(u16);
impl_reflect_value!(u32);
impl_reflect_value!(u64);
impl_reflect_value!(u128);
impl_reflect_value!(f32);
impl_reflect_value!(f64);
impl_reflect_value!(bool);
impl_reflect_value!(String);
impl_reflect_value!(PathBuf);

// ike
impl_reflect_value!(Entity);
