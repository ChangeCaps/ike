use std::{
    any::{type_name, TypeId},
    borrow::Cow,
    collections::BTreeMap,
};

use ike_type::TypeRegistry;

use crate::{Component, Resource, World};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Access {
    Read,
    Write,
}

impl Access {
    pub fn compatible(&self, other: &Self) -> bool {
        match self {
            Self::Read => match other {
                Self::Read => true,
                Self::Write => false,
            },
            Self::Write => false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AccessType {
    Component(TypeId, &'static str),
    Resource(TypeId, &'static str),
}

#[derive(Clone, Default, Debug)]
pub struct SystemAccess {
    pub access: BTreeMap<AccessType, Access>,
}

impl SystemAccess {
    pub fn borrow_component<T: Component>(&mut self, access: Access) {
        self.insert(
            AccessType::Component(TypeId::of::<T>(), type_name::<T>()),
            access,
        );
    }

    pub fn borrow_resource<T: Resource>(&mut self, access: Access) {
        self.insert(
            AccessType::Resource(TypeId::of::<T>(), type_name::<T>()),
            access,
        );
    }

    pub fn insert(&mut self, ty: AccessType, access: Access) {
        if self.access.get(&ty) == Some(&Access::Write) {
            panic!("system access overlap");
        }

        self.access.insert(ty, access);
    }

    pub fn compatible(&self, other: &Self) -> bool {
        for (ty, access) in other.access.iter() {
            if let Some(this_access) = self.access.get(ty) {
                if !this_access.compatible(access) {
                    return false;
                }
            }
        }

        true
    }

    pub fn combine(&mut self, other: Self) {
        for (ty, access) in other.access {
            self.access.insert(ty, access);
        }
    }
}

pub trait System: Send + Sync + 'static {
    fn access(&self) -> SystemAccess;

    fn name(&self) -> &Cow<'static, str>;

    fn run(&mut self, world: &World);

    fn init(&mut self, world: &mut World);

    fn apply(&mut self, world: &mut World);

    fn register_types(&self, type_registry: &mut TypeRegistry);
}
