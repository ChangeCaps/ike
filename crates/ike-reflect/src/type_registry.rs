use std::{
    any::{type_name, Any, TypeId},
    collections::HashMap,
};

use egui::Response;
use erased_serde::Deserializer;
use ike_core::{AnyComponent, Commands, ComponentStorage, Entity};
use serde::de::DeserializeOwned;

use crate::Reflect;

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

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (&TypeId, &TypeRegistration)> {
        self.registrations.iter()
    }
}

#[derive(Clone)]
pub struct TypeRegistration {
    name: &'static str,
    short_name: String,
    type_id: TypeId,
    data: HashMap<TypeId, Box<dyn TypeData>>,
    name_to_id: HashMap<&'static str, TypeId>,
}

impl TypeRegistration {
    #[inline]
    pub fn from_type<T: Send + Sync + 'static>() -> Self {
        Self {
            name: type_name::<T>(),
            short_name: Self::get_short_name(type_name::<T>()),
            type_id: TypeId::of::<T>(),
            data: HashMap::new(),
            name_to_id: HashMap::new(),
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
    pub fn short_name(&self) -> &String {
        &self.short_name
    }

    #[inline]
    pub fn insert<T: TypeData>(&mut self, data: T) {
        self.data.insert(TypeId::of::<T>(), Box::new(data));
        self.name_to_id.insert(type_name::<T>(), TypeId::of::<T>());
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

    #[inline]
    pub unsafe fn data_named<T: TypeData>(&self) -> Option<&T> {
        let id = self.name_to_id.get(type_name::<T>())?;
        if let Some(data) = self.data.get(&id) {
            // SAFETY: TypeIds are the same
            unsafe { Some(&*(data.as_ref() as *const _ as *const _)) }
        } else {
            None
        }
    }

    #[inline]
    pub fn get_short_name(long: &str) -> String {
        let mut short_name = String::new();

        let mut remainder = long;
        while let Some(index) = remainder.find(&['<', '>', '(', ')', '[', ']', ',', ';'] as &[_]) {
            let (path, new_remainder) = remainder.split_at(index);

            short_name.push_str(path.rsplit(':').next().unwrap());

            let ch = new_remainder.chars().next().unwrap();
            short_name.push(ch);

            if ch == ',' || ch == ';' {
                short_name.push(' ');
                remainder = &new_remainder[2..];
            } else {
                remainder = &new_remainder[1..];
            }
        }

        if !remainder.is_empty() {
            short_name.push_str(remainder.rsplit(':').next().unwrap());
        }

        short_name
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
    pub from_storage_mut:
        for<'a> unsafe fn(&'a ComponentStorage, &Entity) -> Option<&'a mut dyn Reflect>,
    pub default: fn() -> Box<dyn Reflect>,
}

impl ReflectComponent {
    #[inline]
    pub fn insert(&self, entity: &Entity, reflect: &dyn Reflect, commands: &Commands) {
        (self.insert)(entity, reflect, commands)
    }

    #[inline]
    pub unsafe fn from_storage_mut<'a>(
        &self,
        storage: &'a ComponentStorage,
        entity: &Entity,
    ) -> Option<&'a mut dyn Reflect> {
        unsafe { (self.from_storage_mut)(storage, entity) }
    }

    #[inline]
    pub fn default(&self) -> Box<dyn Reflect> {
        (self.default)()
    }
}

impl<T: Reflect + AnyComponent> FromType<T> for ReflectComponent {
    #[inline]
    fn from_type() -> Self {
        Self {
            insert: |entity, reflect, commands| {
                if let Some(component) = T::from_reflect(reflect) {
                    commands.insert_component(entity, component);
                }
            },
            from_storage_mut: |storage, entity| unsafe {
                Some(storage.get_unchecked_mut::<T>(entity)?)
            },
            default: || Box::new(T::default_value()),
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

pub trait EguiValue {
    fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response;
}

#[derive(Clone)]
pub struct ReflectEguiValue {
    pub edit: fn(&mut dyn Any, &mut egui::Ui) -> Option<egui::Response>,
}

impl ReflectEguiValue {
    #[inline]
    pub fn edit(&self, value: &mut dyn Any, ui: &mut egui::Ui) -> Option<egui::Response> {
        (self.edit)(value, ui)
    }
}

impl<T: EguiValue + Any> FromType<T> for ReflectEguiValue {
    #[inline]
    fn from_type() -> Self {
        Self {
            edit: |value, ui| {
                if let Some(value) = value.downcast_mut::<T>() {
                    Some(value.ui(ui))
                } else {
                    None
                }
            },
        }
    }
}
