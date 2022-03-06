use std::{any::TypeId, collections::HashMap, mem};

use crate::{AtomicBorrow, Res, ResMut};

pub trait Resource: Send + Sync + 'static {}

impl<T: Send + Sync + 'static> Resource for T {}

struct ResourceBox {
    resource: *mut dyn Resource,
    borrow: AtomicBorrow,
}

impl Drop for ResourceBox {
    fn drop(&mut self) {
        unsafe { Box::from_raw(self.resource) };
    }
}

#[derive(Default)]
pub struct Resources {
    resources: HashMap<TypeId, ResourceBox>,
}

impl Resources {
    pub fn insert<T: Resource>(&mut self, resource: T) {
        self.resources.insert(
            TypeId::of::<T>(),
            ResourceBox {
                resource: Box::into_raw(Box::new(resource)),
                borrow: AtomicBorrow::new(),
            },
        );
    }

    pub fn init<T: Resource + FromResources>(&mut self) {
        if !self.contains::<T>() {
            let resource = T::from_resources(self);

            self.insert(resource);
        }
    }

    pub fn remove<T: Resource>(&mut self) -> Option<T> {
        let resource_box = self.resources.remove(&TypeId::of::<T>())?;

        let resource = unsafe { *Box::from_raw(resource_box.resource as *mut T) };

        mem::forget(resource_box);

        Some(resource)
    }

    pub fn contains<T: Resource>(&self) -> bool {
        self.resources.contains_key(&TypeId::of::<T>())
    }

    pub fn read<'a, T: Resource>(&'a self) -> Option<Res<'a, T>> {
        let resource = self.resources.get(&TypeId::of::<T>())?;

        Res::new(unsafe { &*(resource.resource as *mut T) }, &resource.borrow)
    }

    pub fn write<'a, T: Resource>(&'a self) -> Option<ResMut<'a, T>> {
        let resource = self.resources.get(&TypeId::of::<T>())?;

        ResMut::new(
            unsafe { &mut *(resource.resource as *mut T) },
            &resource.borrow,
        )
    }
}

pub trait FromResources {
    fn from_resources(resources: &Resources) -> Self;
}

impl<T: Default> FromResources for T {
    fn from_resources(_: &Resources) -> Self {
        T::default()
    }
}
