use std::{
    alloc::{alloc, dealloc, Layout},
    any::{type_name, TypeId},
    mem, ptr,
    sync::atomic::{AtomicU64, Ordering},
};

use crate::{AtomicBorrow, Entity, ReadGuard, WriteGuard};

pub trait AnyComponent: Send + Sync + 'static {}

impl<T: Send + Sync + 'static> AnyComponent for T {}

struct ComponentState {
    gen: Option<u64>,
    borrow: AtomicBorrow,
    changed: AtomicU64,
}

impl ComponentState {
    #[inline]
    pub fn new() -> Self {
        Self {
            gen: None,
            borrow: AtomicBorrow::new(),
            changed: AtomicU64::new(0),
        }
    }
}

pub struct ComponentStorage {
    ty: TypeId,
    ty_name: &'static str,
    layout: Layout,
    drop: unsafe fn(*mut u8),
    len: usize,
    cap: usize,
    base: Option<*mut u8>,
    entities: Vec<Entity>,
    component_state: Vec<ComponentState>,
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
            ty_name: type_name::<T>(),
            layout: Layout::new::<T>().pad_to_align(),
            drop: drop_fn::<T>,
            len: 0,
            cap: 0,
            base: None,
            entities: Vec::new(),
            component_state: Vec::new(),
            storage_borrow: AtomicBorrow::new(),
        }
    }

    #[inline]
    pub fn ty(&self) -> TypeId {
        self.ty
    }

    #[inline]
    pub fn ty_name(&self) -> &'static str {
        self.ty_name
    }

    #[inline]
    pub fn contains(&self, entity: &Entity) -> bool {
        self.component_state
            .get(entity.idx() as usize)
            .map_or_else(|| false, |state| state.gen == Some(entity.gen()))
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
        let old_cap = self.cap;
        let new_cap = self.cap + size;

        let layout =
            Layout::from_size_align((self.layout.size() * new_cap).max(1), self.layout.align())
                .unwrap();

        let mem = unsafe { alloc(layout) };

        if let Some(base) = self.base {
            unsafe { ptr::copy_nonoverlapping(base, mem, self.layout.size() * old_cap) };

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
        unsafe { self.base.unwrap().add(self.layout.size() * index) }
    }

    #[inline]
    pub unsafe fn insert_unchecked<T: AnyComponent>(
        &mut self,
        entity: Entity,
        component: T,
        change_frame: u64,
    ) {
        let idx = entity.idx() as usize;

        if self.cap <= idx {
            self.grow(idx - self.cap + 64);

            self.component_state
                .resize_with(self.cap, || ComponentState::new());
        }

        self.len += 1;

        let state = &mut self.component_state[idx];
        state.gen = Some(entity.gen());
        state.borrow = AtomicBorrow::new();
        state.changed = AtomicU64::new(change_frame);

        let ptr = unsafe { self.index_ptr(idx) as *mut T };

        unsafe {
            ptr::copy_nonoverlapping(&component as *const T, ptr, 1);
        }

        mem::forget(component);

        self.entities.push(entity);
    }

    #[inline]
    pub unsafe fn remove_unchecked<T: AnyComponent>(&mut self, entity: &Entity) -> Option<T> {
        if !self.contains(entity) {
            return None;
        }

        let i = self.entities.iter().position(|e| e == entity).unwrap();
        self.entities.remove(i);

        self.len -= 1;

        let idx = entity.idx() as usize;

        self.component_state[idx].gen = None;

        let ptr = unsafe { self.index_ptr(idx) as *mut T };

        let component = unsafe { ptr::read(ptr) };

        Some(component)
    }

    #[inline]
    pub unsafe fn remove_unchecked_raw(&mut self, entity: &Entity) {
        if !self.contains(entity) {
            return;
        }

        let i = self.entities.iter().position(|e| e == entity).unwrap();
        self.entities.remove(i);

        self.len -= 1;

        let idx = entity.idx() as usize;

        self.component_state[idx].gen = None;

        let ptr = unsafe { self.index_ptr(idx) };

        unsafe { (self.drop)(ptr) };
    }

    #[inline]
    pub fn changed(&self, entity: &Entity, last_change_tick: u64, change_tick: u64) -> bool {
        if !self.contains(entity) {
            return false;
        }

        let idx = entity.idx() as usize;

        let this_change_tick = self.component_state[idx].changed.load(Ordering::Acquire);

        let component_delta = change_tick - this_change_tick;
        let system_delta = change_tick - last_change_tick;

        component_delta < system_delta
    }

    #[inline]
    pub fn get_change_marker(&self, entity: &Entity) -> Option<&AtomicU64> {
        if !self.contains(entity) {
            return None;
        }

        let idx = entity.idx() as usize;

        Some(&self.component_state[idx].changed)
    }

    #[inline]
    pub unsafe fn get_raw_unchecked<T: AnyComponent>(&self, entity: &Entity) -> Option<*mut T> {
        if !self.contains(entity) {
            return None;
        }

        let idx = entity.idx() as usize;

        let ptr = unsafe { self.index_ptr(idx) as *mut T };

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
        if !self.contains(entity) {
            return None;
        }

        if !self.borrow() {
            return None;
        }

        let idx = entity.idx() as usize;

        let state = &self.component_state[idx];

        if !state.borrow.borrow() {
            self.release();
            return None;
        }

        let ptr = unsafe { self.index_ptr(idx) as *const T };

        Some(ReadGuard {
            value: unsafe { &*ptr },
            borrow: vec![&state.borrow, &self.storage_borrow],
        })
    }

    #[inline]
    pub unsafe fn get_borrowed_mut<T: AnyComponent>(
        &self,
        entity: &Entity,
    ) -> Option<WriteGuard<T>> {
        if !self.contains(entity) {
            return None;
        }

        if !self.borrow_mut() {
            return None;
        }

        let idx = entity.idx() as usize;

        let state = &self.component_state[idx];

        if !state.borrow.borrow_mut() {
            self.release_mut();
            return None;
        }

        let ptr = unsafe { self.index_ptr(idx) as *mut T };

        Some(WriteGuard {
            value: unsafe { &mut *ptr },
            borrow: vec![&state.borrow, &self.storage_borrow],
            change_detection: None,
        })
    }
}

impl Drop for ComponentStorage {
    #[inline]
    fn drop(&mut self) {
        for (idx, state) in self.component_state.iter().enumerate() {
            if state.gen.is_some() {
                let ptr = unsafe { self.index_ptr(idx) };

                unsafe { (self.drop)(ptr) };
            }
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

        let a = Entity::from_raw(0, 0);
        let b = Entity::from_raw(1, 0);
        unsafe { storage.insert_unchecked(a, 20u64, 0) };
        unsafe { storage.insert_unchecked(b, 32u64, 0) };

        assert_eq!(unsafe { storage.remove_unchecked(&b) }, Some(32));
        assert_eq!(unsafe { storage.remove_unchecked(&a) }, Some(20));
        assert_eq!(unsafe { storage.remove_unchecked::<i64>(&a) }, None);
    }

    #[test]
    fn zero_size_types() {
        struct ZeroSize;

        let mut storage = ComponentStorage::new::<ZeroSize>();

        let a = Entity::from_raw(0, 0);

        unsafe { storage.insert_unchecked(a, ZeroSize, 0) };
    }

    #[test]
    fn unaligned() {
        let mut storage = ComponentStorage::new::<u8>();

        let a = Entity::from_raw(0, 0);

        unsafe { storage.insert_unchecked(a, 127u8, 0) };
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
            let e = Entity::from_raw(i, 0);

            unsafe { storage.insert_unchecked(e, Foo::new(i * 5), 0) };
        }

        for k in 8..16 {
            let e = Entity::from_raw(k, 0);

            assert_eq!(
                unsafe { storage.remove_unchecked(&e) },
                Some(Foo::new(k * 5))
            );
        }

        for k in 8..16 {
            let e = Entity::from_raw(k, 0);

            unsafe { storage.insert_unchecked(e, Foo::new(k * 2), 0) };
        }

        for k in 8..16 {
            let e = Entity::from_raw(k, 0);

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

        let e = Entity::from_raw(0, 0);

        unsafe { storage.insert_unchecked(e, 123i32, 0) };

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

        let e = Entity::from_raw(0, 0);

        unsafe { storage.insert_unchecked(e, 123i32, 0) };

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

        let e = Entity::from_raw(0, 0);

        unsafe { storage.insert_unchecked(e, 123i32, 0) };

        {
            let _k = unsafe { storage.get_borrowed_mut::<i32>(&e) }.unwrap();
            let _k = unsafe { storage.get_borrowed_mut::<i32>(&e) }.unwrap();
        }
    }

    #[test]
    fn borrow_storage() {
        let mut storage = ComponentStorage::new::<i32>();

        let e = Entity::from_raw(0, 0);

        unsafe { storage.insert_unchecked(e, 123i32, 0) };

        storage.borrow();

        assert!(unsafe { storage.get_borrowed::<i32>(&e).is_some() });
        assert!(unsafe { storage.get_borrowed_mut::<i32>(&e).is_none() });

        storage.release();
    }
}
