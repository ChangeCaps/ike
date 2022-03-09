use std::{
    alloc::Layout,
    any::{type_name, TypeId},
    mem, ptr,
};

use crate::{AtomicBorrow, ChangeTick, ComponentTicks};
pub use ike_macro::Component;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComponentStorageKind {
    Sparse,
}

pub trait Component: Sized + Send + Sync + 'static {
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
