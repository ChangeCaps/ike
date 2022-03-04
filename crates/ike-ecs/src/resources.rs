use std::{any::TypeId, collections::HashMap, mem};

use crate::{AtomicBorrow, ResourceRead, ResourceWrite};

pub trait Resource: Send + Sync + 'static {}

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

    pub fn remove<T: Resource>(&mut self) -> Option<T> {
        let resource_box = self.resources.remove(&TypeId::of::<T>())?;

        let resource = unsafe { *Box::from_raw(resource_box.resource as *mut T) };

        mem::forget(resource_box);

        Some(resource)
    }

    pub fn read<'a, T: Resource>(&'a self) -> Option<ResourceRead<'a, T>> {
        let resource = self.resources.get(&TypeId::of::<T>())?;

        ResourceRead::new(unsafe { &*(resource.resource as *mut T) }, &resource.borrow)
    }

    pub fn write<'a, T: Resource>(&'a self) -> Option<ResourceWrite<'a, T>> {
        let resource = self.resources.get(&TypeId::of::<T>())?;

        ResourceWrite::new(
            unsafe { &mut *(resource.resource as *mut T) },
            &resource.borrow,
        )
    }
}
