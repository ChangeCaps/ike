use std::{
    any::{type_name, TypeId},
    collections::HashMap,
};

use erased_serde::Deserializer;
use ike_core::{AnyComponent, Commands, Entity};
use serde::de::DeserializeOwned;

use crate::{FromReflect, Reflect};

pub trait RegisterType {
    fn register(type_registry: &mut TypeRegistry);
}

#[derive(Default)]
pub struct TypeRegistry {
    registrations: HashMap<TypeId, TypeRegistration>,
    name_to_id: HashMap<&'static str, TypeId>,
}

impl TypeRegistry {
    #[inline]
    pub fn register<T: RegisterType>(&mut self) {
        T::register(self)
    }

    #[inline]
    pub fn insert(&mut self, registration: TypeRegistration) {
        self.name_to_id
            .insert(registration.name(), registration.type_id());
        self.registrations
            .insert(registration.type_id(), registration);
    }

    #[inline]
    pub fn contains(&self, type_id: &TypeId) -> bool {
        self.registrations.contains_key(type_id)
    }

    #[inline]
    pub fn get(&self, type_id: &TypeId) -> Option<&TypeRegistration> {
        self.registrations.get(type_id)
    }

    #[inline]
    pub fn get_name(&self, name: impl AsRef<str>) -> Option<&TypeRegistration> {
        let id = self.name_to_id.get(name.as_ref())?;
        self.get(id)
    }
}

#[derive(Clone)]
pub struct TypeRegistration {
    name: &'static str,
    type_id: TypeId,
    data: HashMap<TypeId, Box<dyn TypeData>>,
}

impl TypeRegistration {
    #[inline]
    pub fn from_type<T: Send + Sync + 'static>() -> Self {
        Self {
            name: type_name::<T>(),
            type_id: TypeId::of::<T>(),
            data: HashMap::new(),
        }
    }

    #[inline]
    pub fn type_id(&self) -> TypeId {
        self.type_id
    }

    #[inline]
    pub fn name(&self) -> &'static str {
        self.name
    }

    #[inline]
    pub fn insert<T: TypeData>(&mut self, data: T) {
        self.data.insert(TypeId::of::<T>(), Box::new(data));
    }

    #[inline]
    pub fn data<T: TypeData>(&self) -> Option<&T> {
        if let Some(data) = self.data.get(&TypeId::of::<T>()) {
            // SAFETY: TypeIds are the same
            unsafe { Some(&*(data.as_ref() as *const _ as *const _)) }
        } else {
            None
        }
    }

    #[inline]
    pub fn data_mut<T: TypeData>(&mut self) -> Option<&mut T> {
        if let Some(data) = self.data.get_mut(&TypeId::of::<T>()) {
            // SAFETY: TypeIds are the same
            unsafe { Some(&mut *(data.as_mut() as *mut _ as *mut _)) }
        } else {
            None
        }
    }
}

pub trait TypeData: Send + Sync + 'static {
    fn clone_type_data(&self) -> Box<dyn TypeData>;
}

impl<T: 'static + Send + Sync + Clone> TypeData for T {
    #[inline]
    fn clone_type_data(&self) -> Box<dyn TypeData> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn TypeData> {
    #[inline]
    fn clone(&self) -> Self {
        self.clone_type_data()
    }
}

pub trait FromType<T> {
    fn from_type() -> Self;
}

#[derive(Clone)]
pub struct ReflectComponent {
    pub insert: fn(&Entity, &dyn Reflect, commands: &Commands),
}

impl ReflectComponent {
    #[inline]
    pub fn insert(&self, entity: &Entity, reflect: &dyn Reflect, commands: &Commands) {
        (self.insert)(entity, reflect, commands)
    }
}

impl<T: Reflect + AnyComponent + FromReflect> FromType<T> for ReflectComponent {
    #[inline]
    fn from_type() -> Self {
        Self {
            insert: |entity, reflect, commands| {
                if let Some(component) = T::from_reflect(reflect) {
                    commands.insert_component(entity, component);
                }
            },
        }
    }
}

#[derive(Clone)]
pub struct ReflectDeserialize {
    pub deserialize: fn(&mut dyn Deserializer) -> Result<Box<dyn Reflect>, erased_serde::Error>,
}

impl ReflectDeserialize {
    #[inline]
    pub fn deserialize(
        &self,
        deserializer: &mut dyn Deserializer,
    ) -> Result<Box<dyn Reflect>, erased_serde::Error> {
        (self.deserialize)(deserializer)
    }
}

impl<T: Reflect + DeserializeOwned> FromType<T> for ReflectDeserialize {
    #[inline]
    fn from_type() -> Self {
        Self {
            deserialize: |deserializer| {
                T::deserialize(deserializer).map(|value| Box::new(value) as Box<dyn Reflect>)
            },
        }
    }
}
