use std::{alloc::Layout, any::TypeId, collections::BTreeMap, mem, ptr};

use crate::{AtomicBorrow, Entity, ReadGuard, WriteGuard};

pub trait AnyComponent: Send + Sync + 'static {}

impl<T: Send + Sync + 'static> AnyComponent for T {}

pub struct ComponentStorage {
    ty: TypeId,
    layout: Layout,
    drop: unsafe fn(*mut u8),
    entities: BTreeMap<Entity, usize>,
    data: Vec<u8>,
    unused_data: Vec<usize>,
    borrow: Vec<AtomicBorrow>,
}

impl ComponentStorage {
    #[inline]
    pub fn new<T: AnyComponent>() -> Self {
        unsafe fn drop_fn<T>(component: *mut u8) {
            unsafe { std::ptr::drop_in_place(component as *mut T) }
        }

        Self {
            ty: TypeId::of::<T>(),
            layout: Layout::new::<T>().pad_to_align(),
            drop: drop_fn::<T>,
            entities: BTreeMap::new(),
            data: Vec::new(),
            unused_data: Vec::new(),
            borrow: Vec::new(),
        }
    }

    #[inline]
    pub fn ty(&self) -> TypeId {
        self.ty
    }

    #[inline]
    pub unsafe fn insert_unchecked<T: AnyComponent>(&mut self, entity: Entity, component: T) {
        if self.entities.contains_key(&entity) {
            return;
        }

        let idx = if let Some(idx) = self.unused_data.pop() {
            self.borrow[idx / self.layout.size()] = AtomicBorrow::new();

            idx
        } else {
            let len = self.data.len();

            self.data.resize(len + self.layout.size(), 0);
            self.borrow.push(AtomicBorrow::new());

            len
        };

        let ptr = unsafe { &mut *(&mut self.data[idx] as *mut _ as *mut T) };

        mem::forget(mem::replace(ptr, component));

        self.entities.insert(entity, idx);
    }

    #[inline]
    pub unsafe fn remove_unchecked<T: AnyComponent>(&mut self, entity: &Entity) -> Option<T> {
        let idx = self.entities.remove(entity)?;
        self.unused_data.push(idx);

        let ptr = &mut self.data[idx] as *mut _ as *mut T;

        let component = unsafe { ptr::read(ptr) };

        Some(component)
    }

    #[inline]
    pub unsafe fn get_raw_unchecked<T: AnyComponent>(&self, entity: &Entity) -> Option<*mut T> {
        let idx = self.entities.get(entity)?;

        let ptr = &self.data[*idx] as *const _ as *mut T;

        Some(ptr)
    }

    #[inline]
    pub unsafe fn get_unchecked<T: AnyComponent>(&self, entity: &Entity) -> Option<ReadGuard<T>> {
        let idx = self.entities.get(entity)?;

        let borrow = &self.borrow[*idx / self.layout.size()];

        if !borrow.borrow() {
            return None;
        }

        let ptr = &self.data[*idx] as *const _ as *mut T;

        Some(ReadGuard {
            value: unsafe { &*ptr },
            borrow,
        })
    }

    #[inline]
    pub unsafe fn get_unchecked_mut<T: AnyComponent>(
        &self,
        entity: &Entity,
    ) -> Option<WriteGuard<T>> {
        let idx = self.entities.get(entity)?;

        let borrow = &self.borrow[*idx / self.layout.size()];

        if !borrow.borrow_mut() {
            return None;
        }

        let ptr = &self.data[*idx] as *const _ as *mut T;

        Some(WriteGuard {
            value: unsafe { &mut *ptr },
            borrow,
        })
    }
}

impl Drop for ComponentStorage {
    #[inline]
    fn drop(&mut self) {
        for idx in self.entities.values() {
            let ptr = &mut self.data[*idx] as *mut u8;

            unsafe { (self.drop)(ptr) };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_storage() {
        let mut storage = ComponentStorage::new::<i64>();

        let a = Entity::from_raw(0);
        let b = Entity::from_raw(1);
        unsafe { storage.insert_unchecked(a, 20u64) };
        unsafe { storage.insert_unchecked(b, 32u64) };

        assert_eq!(unsafe { storage.remove_unchecked(&b) }, Some(32));
        assert_eq!(unsafe { storage.remove_unchecked(&a) }, Some(20));
        assert_eq!(unsafe { storage.remove_unchecked::<i64>(&a) }, None);
    }

    #[test]
    fn advanced_storage() {
        #[derive(Debug, Default, PartialEq, Eq)]
        struct Foo {
            h: bool,
            j: u8,
            b: std::sync::Arc<u64>,
            z: i32,
        }

        impl Foo {
            fn new(b: u64) -> Self {
                Self {
                    h: false,
                    j: 127,
                    b: b.into(),
                    z: -12739812,
                }
            }
        }

        let mut storage = ComponentStorage::new::<Foo>();

        for i in 0..32 {
            let e = Entity::from_raw(i);

            unsafe { storage.insert_unchecked(e, Foo::new(i * 5)) };
        }

        for k in 8..16 {
            let e = Entity::from_raw(k);

            assert_eq!(
                unsafe { storage.remove_unchecked(&e) },
                Some(Foo::new(k * 5))
            );
        }

        for k in 8..16 {
            let e = Entity::from_raw(k);

            unsafe { storage.insert_unchecked(e, Foo::new(k * 2)) };
        }

        for k in 8..16 {
            let e = Entity::from_raw(k);

            assert_eq!(
                unsafe { storage.remove_unchecked(&e) },
                Some(Foo::new(k * 2))
            );
        }
    }

    #[test]
    #[should_panic]
    fn mut_after_borrow() {
        let mut storage = ComponentStorage::new::<i32>();

        let e = Entity::from_raw(0);

        unsafe { storage.insert_unchecked(e, 123i32) };

        {
            let _k = unsafe { storage.get_unchecked::<i32>(&e) }.unwrap();
            let _k = unsafe { storage.get_unchecked::<i32>(&e) }.unwrap();
            let _k = unsafe { storage.get_unchecked_mut::<i32>(&e) }.unwrap();
        }
    }

    #[test]
    #[should_panic]
    fn borrow_after_mut() {
        let mut storage = ComponentStorage::new::<i32>();

        let e = Entity::from_raw(0);

        unsafe { storage.insert_unchecked(e, 123i32) };

        {
            let _k = unsafe { storage.get_unchecked_mut::<i32>(&e) }.unwrap();
            let _k = unsafe { storage.get_unchecked::<i32>(&e) }.unwrap();
            let _k = unsafe { storage.get_unchecked::<i32>(&e) }.unwrap();
        }
    }

    #[test]
    #[should_panic]
    fn mut_after_mut() {
        let mut storage = ComponentStorage::new::<i32>();

        let e = Entity::from_raw(0);

        unsafe { storage.insert_unchecked(e, 123i32) };

        {
            let _k = unsafe { storage.get_unchecked_mut::<i32>(&e) }.unwrap();
            let _k = unsafe { storage.get_unchecked_mut::<i32>(&e) }.unwrap();
        }
    }
}
