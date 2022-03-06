use std::{any::TypeId, collections::BTreeMap};

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

    pub fn max(self, other: Self) -> Self {
        match self {
            Self::Read => match other {
                Self::Read => Self::Read,
                Self::Write => Self::Write,
            },
            Self::Write => Self::Write,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AccessType {
    Component(TypeId),
    Resource(TypeId),
}

#[derive(Clone, Default, Debug)]
pub struct SystemAccess {
    pub access: BTreeMap<AccessType, Access>,
    pub world: bool,
}

impl SystemAccess {
    pub fn borrow_component<T: Component>(&mut self, access: Access) {
        self.access
            .insert(AccessType::Component(TypeId::of::<T>()), access);
    }

    pub fn borrow_resource<T: Resource>(&mut self, access: Access) {
        self.access
            .insert(AccessType::Component(TypeId::of::<T>()), access);
    }

    pub fn borrow_world(&mut self) {
        self.world = true;
    }

    pub fn compatible(&self, other: &Self) -> bool {
        if self.world {
            if other.world {
                return false;
            }

            if other.access.len() > 0 {
                return false;
            }
        }

        if other.world && self.access.len() > 0 {
            return false;
        }

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
        self.world |= other.world;

        for (ty, access) in other.access {
            if let Some(this_access) = self.access.get_mut(&ty) {
                *this_access = (*this_access).max(access);
            } else {
                self.access.insert(ty, access);
            }
        }
    }
}

pub trait ExclusiveSystem: Send + Sync + 'static {
    fn run(&mut self, world: &mut World);
}

impl<F: FnMut(&mut World) + Send + Sync + 'static> ExclusiveSystem for F {
    fn run(&mut self, world: &mut World) {
        world.increment_change_tick();

        self(world);
    }
}

pub trait System: Send + Sync + 'static {
    fn access(&self) -> SystemAccess;

    fn name(&self) -> &str;

    fn run(&mut self, world: &World);

    fn init(&mut self, world: &mut World);

    fn apply(&mut self, world: &mut World);
}
