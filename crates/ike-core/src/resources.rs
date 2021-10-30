use std::{any::TypeId, collections::HashMap};

use crate::{BorrowLock, ReadGuard, WriteGuard};

pub trait Resource: Send + Sync + 'static {}

impl<T: Send + Sync + 'static> Resource for T {}

#[derive(Default)]
pub struct Resources {
    inner: HashMap<TypeId, BorrowLock<dyn Resource>>,
}

impl Resources {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn insert<T: Resource>(&mut self, resource: T) {
        self.inner.insert(
            TypeId::of::<T>(),
            BorrowLock::new(resource).map(|ptr| ptr as *mut _),
        );
    }

    #[inline]
    pub fn read<T: Resource>(&self) -> Option<ReadGuard<T>> {
        Some(
            self.inner
                .get(&TypeId::of::<T>())?
                .read()?
                .map(|resource| unsafe { &*(resource as *const _ as *const _) }),
        )
    }

    #[inline]
    pub fn write<T: Resource>(&self) -> Option<WriteGuard<T>> {
        let write = self.inner.get(&TypeId::of::<T>())?.write()?;

        Some(WriteGuard {
            value: unsafe { &mut *(write.value as *mut _ as *mut _) },
            borrow: write.borrow,
        })
    }
}
