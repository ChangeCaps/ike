use std::{
    alloc::Layout,
    any::{type_name, TypeId},
    mem, ptr,
};

use crate::{AtomicBorrow, ChangeTick, Comp, CompMut, ComponentTicks, Entity, World};
pub use ike_macro::component;
use ike_reflect::Reflect;
use ike_type::{FromType, Registerable};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComponentStorageKind {
    Sparse,
}

pub trait Component: Registerable + Sized + Send + Sync + 'static {
    type Storage: ComponentStorage;
}

pub trait ComponentStorage {
    const STORAGE: ComponentStorageKind;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SparseStorage;

impl ComponentStorage for SparseStorage {
    const STORAGE: ComponentStorageKind = ComponentStorageKind::Sparse;
}

#[derive(Clone)]
pub struct ReflectComponent {
    get: for<'a> fn(&'a World, &Entity) -> Option<Comp<'a, dyn Reflect>>,
    get_mut: for<'a> fn(&'a World, &Entity) -> Option<CompMut<'a, dyn Reflect>>,
}

impl ReflectComponent {
    pub fn get<'a>(&self, world: &'a World, entity: &Entity) -> Option<Comp<'a, dyn Reflect>> {
        (self.get)(world, entity)
    }

    pub fn get_mut<'a>(
        &self,
        world: &'a World,
        entity: &Entity,
    ) -> Option<CompMut<'a, dyn Reflect>> {
        (self.get_mut)(world, entity)
    }
}

impl<T: Component + Reflect> FromType<T> for ReflectComponent {
    fn from_type() -> Self {
        Self {
            get: |world, entity| Some(world.component::<T>(entity)?.map_inner(|r| r as _)),
            get_mut: |world, entity| Some(world.component_mut::<T>(entity)?.map_inner(|r| r as _)),
        }
    }
}

pub struct ComponentData {
    pub ticks: ComponentTicks,
    pub borrow: AtomicBorrow,
}

impl ComponentData {
    pub const fn new(change_tick: ChangeTick) -> Self {
        Self {
            ticks: ComponentTicks::new(change_tick),
            borrow: AtomicBorrow::new(),
        }
    }
}

/// An untyped description of a [`Component`].
pub struct ComponentDescriptor {
    pub type_name: &'static str,
    pub type_id: TypeId,
    pub layout: Layout,
    pub drop: unsafe fn(*mut u8),
    pub needs_drop: bool,
}

impl ComponentDescriptor {
    pub fn new<T: Component>() -> Self {
        Self {
            type_name: type_name::<T>(),
            type_id: TypeId::of::<T>(),
            layout: Layout::new::<T>(),
            drop: |ptr| unsafe { ptr::drop_in_place(ptr as *mut T) },
            needs_drop: mem::needs_drop::<T>(),
        }
    }
}
