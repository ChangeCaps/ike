use std::{
    alloc::{alloc, dealloc, Layout},
    any::TypeId,
    collections::BTreeMap,
    mem, ptr,
};

use crate::{AtomicBorrow, Entity, ReadGuard, WriteGuard};

pub trait AnyComponent: Send + Sync + 'static {}

impl<T: Send + Sync + 'static> AnyComponent for T {}

pub struct ComponentStorage {
    ty: TypeId,
    layout: Layout,
    drop: unsafe fn(*mut u8),
    entities: Vec<Entity>,
    entity_indices: BTreeMap<Entity, usize>,
    len: usize,
    cap: usize,
    base: Option<*mut u8>,
    unused_data: Vec<usize>,
    component_borrow: Vec<AtomicBorrow>,
    storage_borrow: AtomicBorrow,
}

unsafe impl Send for ComponentStorage {}
unsafe impl Sync for ComponentStorage {}

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
            entities: Vec::new(),
            entity_indices: BTreeMap::new(),
            len: 0,
            cap: 0,
            base: None,
            unused_data: Vec::new(),
            component_borrow: Vec::new(),
            storage_borrow: AtomicBorrow::new(),
        }
    }

    #[inline]
    pub fn ty(&self) -> TypeId {
        self.ty
    }

    #[inline]
    pub fn contains(&self, entity: &Entity) -> bool {
        self.entity_indices.contains_key(entity)
    }

    #[inline]
    pub fn borrow(&self) -> bool {
        self.storage_borrow.borrow()
    }

    #[inline]
    pub fn borrow_mut(&self) -> bool {
        self.storage_borrow.borrow_mut()
    }

    #[inline]
    pub fn release(&self) {
        self.storage_borrow.release();
    }

    #[inline]
    pub fn release_mut(&self) {
        self.storage_borrow.release_mut();
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.cap
    }

    #[inline]
    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }

    #[inline]
    pub fn grow(&mut self, min_size: usize) {
        self.grow_exact(self.capacity().max(min_size));
    }

    #[inline]
    pub fn grow_exact(&mut self, size: usize) {
        let old_len = self.len;
        let old_cap = self.cap;
        let new_cap = self.cap + size;

        let layout =
            Layout::from_size_align((self.layout.size() * new_cap).max(1), self.layout.align())
                .unwrap();

        let mem = unsafe { alloc(layout) };

        if let Some(base) = self.base {
            unsafe { ptr::copy_nonoverlapping(base, mem, self.layout.size() * old_len) };

            let layout =
                Layout::from_size_align((self.layout.size() * old_cap).max(1), self.layout.align())
                    .unwrap();

            unsafe { dealloc(base, layout) };
        }

        self.base = Some(mem);

        self.cap = new_cap;
    }

    #[inline]
    pub unsafe fn get_base<T: AnyComponent>(&self) -> Option<*mut T> {
        self.base.map(|base| base as *mut T)
    }

    #[inline]
    pub unsafe fn index_ptr(&self, index: usize) -> *mut u8 {
        if index >= self.len() {
            panic!("index must be < len");
        }

        unsafe { self.base.unwrap().add(self.layout.size() * index) }
    }

    #[inline]
    pub unsafe fn insert_unchecked<T: AnyComponent>(&mut self, entity: Entity, component: T) {
        if self.entity_indices.contains_key(&entity) {
            return;
        }

        let idx = if let Some(idx) = self.unused_data.pop() {
            self.component_borrow[idx] = AtomicBorrow::new();

            idx
        } else {
            if self.len == self.cap {
                self.grow(64);
            }

            self.component_borrow.push(AtomicBorrow::new());
            self.entities.push(entity);

            let idx = self.len;

            self.len += 1;

            idx
        };

        let ptr = unsafe { self.index_ptr(idx) as *mut T };

        unsafe {
            ptr::copy_nonoverlapping(&component as *const T, ptr, 1);
        }

        mem::forget(component);

        self.entity_indices.insert(entity, idx);
    }

    #[inline]
    pub unsafe fn remove_unchecked<T: AnyComponent>(&mut self, entity: &Entity) -> Option<T> {
        self.entities.retain(|e| e != entity);

        let idx = self.entity_indices.remove(entity)?;
        self.unused_data.push(idx);

        let ptr = unsafe { self.index_ptr(idx) as *mut T };

        let component = unsafe { ptr::read(ptr) };

        Some(component)
    }

    #[inline]
    pub unsafe fn get_raw_unchecked<T: AnyComponent>(&self, entity: &Entity) -> Option<*mut T> {
        let idx = self.entity_indices.get(entity)?;

        let ptr = unsafe { self.index_ptr(*idx) as *mut T };

        Some(ptr)
    }

    #[inline]
    pub unsafe fn get_unchecked<T: AnyComponent>(&self, entity: &Entity) -> Option<&T> {
        Some(unsafe { &*self.get_raw_unchecked(entity)? })
    }

    #[inline]
    pub unsafe fn get_unchecked_mut<T: AnyComponent>(&self, entity: &Entity) -> Option<&mut T> {
        Some(unsafe { &mut *self.get_raw_unchecked(entity)? })
    }

    #[inline]
    pub unsafe fn get_borrowed<T: AnyComponent>(&self, entity: &Entity) -> Option<ReadGuard<T>> {
        if !self.borrow() {
            return None;
        }

        let idx = self.entity_indices.get(entity)?;

        let borrow = &self.component_borrow[*idx];

        if !borrow.borrow() {
            return None;
        }

        let ptr = unsafe { self.index_ptr(*idx) as *const T };

        Some(ReadGuard {
            value: unsafe { &*ptr },
            borrow: vec![borrow, &self.storage_borrow],
        })
    }

    #[inline]
    pub unsafe fn get_borrowed_mut<T: AnyComponent>(
        &self,
        entity: &Entity,
    ) -> Option<WriteGuard<T>> {
        if !self.borrow_mut() {
            return None;
        }

        let idx = self.entity_indices.get(entity)?;

        let borrow = &self.component_borrow[*idx];

        if !borrow.borrow_mut() {
            return None;
        }

        let ptr = unsafe { self.index_ptr(*idx) as *mut T };

        Some(WriteGuard {
            value: unsafe { &mut *ptr },
            borrow: vec![borrow, &self.storage_borrow],
        })
    }
}

impl Drop for ComponentStorage {
    #[inline]
    fn drop(&mut self) {
        for idx in self.entity_indices.values() {
            let ptr = unsafe { self.index_ptr(*idx) };

            unsafe { (self.drop)(ptr) };
        }

        if let Some(base) = self.base {
            let layout = Layout::from_size_align(
                (self.layout.size() * self.cap).max(1),
                self.layout.align(),
            )
            .unwrap();

            unsafe { dealloc(base, layout) };
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
    fn zero_size_types() {
        struct ZeroSize;

        let mut storage = ComponentStorage::new::<ZeroSize>();

        let a = Entity::from_raw(0);

        unsafe { storage.insert_unchecked(a, ZeroSize) };
    }

    #[test]
    fn unaligned() {
        let mut storage = ComponentStorage::new::<u8>();

        let a = Entity::from_raw(0);

        unsafe { storage.insert_unchecked(a, 127u8) };
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
            let _k = unsafe { storage.get_borrowed::<i32>(&e) }.unwrap();
            let _k = unsafe { storage.get_borrowed::<i32>(&e) }.unwrap();
            let _k = unsafe { storage.get_borrowed_mut::<i32>(&e) }.unwrap();
        }
    }

    #[test]
    #[should_panic]
    fn borrow_after_mut() {
        let mut storage = ComponentStorage::new::<i32>();

        let e = Entity::from_raw(0);

        unsafe { storage.insert_unchecked(e, 123i32) };

        {
            let _k = unsafe { storage.get_borrowed_mut::<i32>(&e) }.unwrap();
            let _k = unsafe { storage.get_borrowed::<i32>(&e) }.unwrap();
            let _k = unsafe { storage.get_borrowed::<i32>(&e) }.unwrap();
        }
    }

    #[test]
    #[should_panic]
    fn mut_after_mut() {
        let mut storage = ComponentStorage::new::<i32>();

        let e = Entity::from_raw(0);

        unsafe { storage.insert_unchecked(e, 123i32) };

        {
            let _k = unsafe { storage.get_borrowed_mut::<i32>(&e) }.unwrap();
            let _k = unsafe { storage.get_borrowed_mut::<i32>(&e) }.unwrap();
        }
    }

    #[test]
    fn borrow_storage() {
        let mut storage = ComponentStorage::new::<i32>();

        let e = Entity::from_raw(0);

        unsafe { storage.insert_unchecked(e, 123i32) };

        storage.borrow();

        assert!(unsafe { storage.get_borrowed::<i32>(&e).is_some() });
        assert!(unsafe { storage.get_borrowed_mut::<i32>(&e).is_none() });

        storage.release();
    }
}
