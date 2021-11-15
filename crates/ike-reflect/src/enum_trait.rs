use std::any::Any;

use crate::{Reflect, ReflectMut, ReflectRef, Struct, TupleStruct};

pub enum EnumVariant<'a> {
    Struct(&'a dyn Struct),
    TupleStruct(&'a dyn TupleStruct),
    Unit,
}

pub enum EnumVariantMut<'a> {
    Struct(&'a mut dyn Struct),
    TupleStruct(&'a dyn TupleStruct),
    Unit,
}

pub trait Enum: Reflect {
    fn variant_name(&self) -> &str;
    fn variant_value(&self) -> EnumVariant;
    fn clone_dynamic(&self) -> DynamicEnum;
}

pub enum DynamicEnumVariant {
    Struct(Box<dyn Reflect>),
    TupleStruct(Box<dyn Reflect>),
    Unit,
}

impl DynamicEnumVariant {
    #[inline]
    pub fn enum_variant(&self) -> EnumVariant {
        match self {
            Self::Struct(value) => EnumVariant::Struct(match value.reflect_ref() {
                ReflectRef::Struct(value) => value,
                _ => unreachable!(),
            }),
            Self::TupleStruct(value) => EnumVariant::TupleStruct(match value.reflect_ref() {
                ReflectRef::TupleStruct(value) => value,
                _ => unreachable!(),
            }),
            Self::Unit => EnumVariant::Unit,
        }
    }

    #[inline]
    pub fn enum_variant_mut(&mut self) -> EnumVariantMut {
        match self {
            Self::Struct(value) => EnumVariantMut::Struct(match value.reflect_mut() {
                ReflectMut::Struct(value) => value,
                _ => unreachable!(),
            }),
            Self::TupleStruct(value) => EnumVariantMut::TupleStruct(match value.reflect_mut() {
                ReflectMut::TupleStruct(value) => value,
                _ => unreachable!(),
            }),
            Self::Unit => EnumVariantMut::Unit,
        }
    }

    #[inline]
    pub fn clone_dynamic(&self) -> DynamicEnumVariant {
        match self {
            Self::Struct(value) => Self::Struct(value.clone_value()),
            Self::TupleStruct(value) => Self::TupleStruct(value.clone_value()),
            Self::Unit => Self::Unit,
        }
    }
}

pub struct DynamicEnum {
    name: String,
    variant_name: String,
    variant_value: DynamicEnumVariant,
}

impl DynamicEnum {
    #[inline]
    pub fn new(variant: DynamicEnumVariant) -> Self {
        Self {
            name: Default::default(),
            variant_name: Default::default(),
            variant_value: variant,
        }
    }

    #[inline]
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    #[inline]
    pub fn set_variant_name(&mut self, name: impl Into<String>) {
        self.variant_name = name.into();
    }
}

/*
impl Enum for DynamicEnum {
    #[inline]
    fn variant_name(&self) -> &str {
        &self.variant_name
    }

    #[inline]
    fn variant_value(&self) -> EnumVariant {
        self.variant_value.enum_variant()
    }

    #[inline]
    fn clone_dynamic(&self) -> DynamicEnum {
        Self {
            name: self.name.clone(),
            variant_name: self.variant_name.clone(),
            variant_value: self.variant_value.clone_dynamic(),
        }
    }
}

unsafe impl Reflect for DynamicEnum {
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
        ReflectRef::Enum(self)
    }

    #[inline]
    fn reflect_mut(&mut self) -> ReflectMut {
        ReflectMut::Enum(self)
    }

    #[inline]
    fn clone_value(&self) -> Box<dyn Reflect> {
        Box::new(self.clone_dynamic())
    }
}
*/
