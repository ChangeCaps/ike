use std::{
    any::{type_name, TypeId},
    collections::HashMap,
};

use crate::{BorrowLock, ReadGuard, WriteGuard};

pub trait Resource: Send + Sync + 'static {}

impl<T: Send + Sync + 'static> Resource for T {}

#[derive(Default)]
pub struct Resources {
    inner: HashMap<TypeId, BorrowLock<dyn Resource>>,
    type_ids: HashMap<&'static str, TypeId>,
}

impl Resources {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn remove_raw(&mut self, type_id: TypeId) {
        self.inner.remove(&type_id);
    }

    #[inline]
    pub fn contains_raw(&self, type_id: TypeId) -> bool {
        self.inner.contains_key(&type_id)
    }

    #[inline]
    pub fn insert<T: Resource>(&mut self, resource: T) {
        self.inner
            .insert(TypeId::of::<T>(), BorrowLock::from_box(Box::new(resource)));
        self.type_ids.insert(type_name::<T>(), TypeId::of::<T>());
    }

    #[inline]
    pub fn contains<T: Resource>(&self) -> bool {
        self.inner.contains_key(&TypeId::of::<T>())
    }

    #[inline]
    pub fn remove<T: Resource>(&mut self) -> Option<T> {
        self.type_ids.remove(type_name::<T>());
        let resource = self.inner.remove(&TypeId::of::<T>())?;

        Some(unsafe { *Box::from_raw(resource.into_raw() as *mut T) })
    }

    #[inline]
    pub fn read<T: Resource>(&self) -> Option<ReadGuard<T>> {
        let read = self.inner.get(&TypeId::of::<T>())?.read()?;

        let out = ReadGuard {
            value: unsafe { &*(read.value as *const dyn Resource as *const T) },
            borrow: read.forget(),
        };

        Some(out)
    }

    #[inline]
    pub fn write<T: Resource>(&self) -> Option<WriteGuard<T>> {
        let mut write = self.inner.get(&TypeId::of::<T>())?.write()?;

        Some(WriteGuard {
            value: unsafe { &mut *(write.value as *mut _ as *mut _) },
            change_detection: write.change_detection.take(),
            borrow: write.forget(),
        })
    }

    #[inline]
    pub unsafe fn read_named<T: Resource>(&self) -> Option<ReadGuard<T>> {
        let type_id = self.type_ids.get(type_name::<T>())?;
        let read = self.inner.get(type_id)?.read()?;

        let out = ReadGuard {
            value: unsafe { &*(read.value as *const dyn Resource as *const T) },
            borrow: read.forget(),
        };

        Some(out)
    }

    #[inline]
    pub unsafe fn write_named<T: Resource>(&self) -> Option<WriteGuard<T>> {
        let type_id = self.type_ids.get(type_name::<T>())?;
        let mut write = self.inner.get(type_id)?.write()?;

        Some(WriteGuard {
            value: unsafe { &mut *(write.value as *mut _ as *mut _) },
            change_detection: write.change_detection.take(),
            borrow: write.forget(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resources() {
        let mut resources = Resources::new();

        resources.insert(123);

        resources.read::<i32>().unwrap();
    }
}
