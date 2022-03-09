use std::{
    alloc::{self, handle_alloc_error, Layout},
    mem::MaybeUninit,
    ptr::{self, NonNull},
};

use crate::{ChangeTick, ComponentData, ComponentDescriptor, Entity, EntitySet};

/// Sparse component storage.
pub struct SparseComponentStorage {
    component_layout: Layout,
    drop: unsafe fn(*mut u8),
    needs_drop: bool,
    capacity: usize,
    entities: EntitySet,
    component_data: Vec<MaybeUninit<ComponentData>>,
    data: NonNull<u8>,
}

unsafe impl Send for SparseComponentStorage {}
unsafe impl Sync for SparseComponentStorage {}

impl SparseComponentStorage {
    pub fn new(desc: &ComponentDescriptor) -> Self {
        Self {
            component_layout: desc.layout,
            drop: desc.drop,
            needs_drop: desc.needs_drop,
            capacity: 0,
            entities: EntitySet::new(),
            component_data: Vec::new(),
            data: NonNull::dangling(),
        }
    }

    pub fn clear(&mut self) {
        self.entities.clear();
    }

    pub fn entities(&self) -> &EntitySet {
        &self.entities
    }

    pub const fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn contains(&self, entity: &Entity) -> bool {
        self.entities.contains(entity)
    }

    /// # Safety
    /// - item pointed to by `component` must be a valid input for drop in the descriptor
    /// used to create self.
    /// - item pointed to by `component` must have the same layout as described by the
    /// descriptor used to create self.
    /// - item pointed to by `component` will be moved by calling this function
    /// and should therefore not be used or dropped.
    /// - `component` must not point to item at `entity.index()` in self.
    pub unsafe fn insert_unchecked(
        &mut self,
        entity: Entity,
        component: *mut u8,
        change_tick: ChangeTick,
    ) {
        debug_assert!(!self.contains(&entity));

        let index = entity.index() as usize;

        if self.capacity <= index {
            self.grow_exact(index - self.capacity + 1);
        }

        // SAFETY:
        // the above statement ensures that self.capacity is >= index
        let ptr = unsafe { self.get_unchecked(index) };

        unsafe { ptr::copy_nonoverlapping(component, ptr, self.component_layout.size()) };

        self.entities.insert(entity);

        self.component_data[index] = MaybeUninit::new(ComponentData::new(change_tick));
    }

    /// # Safety
    /// - `entity.index()` must be < `self.capacity`.
    pub unsafe fn remove_unchecked(&mut self, entity: &Entity) -> *mut u8 {
        self.entities.remove(entity);

        unsafe { self.get_unchecked(entity.index() as usize) }
    }

    /// # Safety
    /// - `entity.index()` must be < `self.capacity`.
    /// - component at `entity.index()` in self must be valid.
    pub unsafe fn drop_unchecked(&mut self, entity: &Entity) {
        let ptr = unsafe { self.remove_unchecked(entity) };

        unsafe { (self.drop)(ptr) };
    }

    /// # Safety
    /// - `index` must be < `self.capacity`.
    pub unsafe fn get_unchecked(&self, index: usize) -> *mut u8 {
        debug_assert!(index < self.capacity());
        unsafe { self.data.as_ptr().add(index * self.component_layout.size()) }
    }

    /// # Safety
    /// - `index` must be < `self.capacity`.
    /// - component at index must be valid
    pub unsafe fn get_data_unchecked(&self, index: usize) -> &ComponentData {
        unsafe { self.component_data.get_unchecked(index).assume_init_ref() }
    }

    fn grow_exact(&mut self, additional: usize) {
        let new_capacity = self.capacity + additional;

        if self.component_layout.size() > 0 {
            let new_layout = repeat_layout(&self.component_layout, new_capacity)
                .expect("array layout should be vaild");

            let new_data = if self.capacity == 0 {
                unsafe { alloc::alloc(new_layout) }
            } else {
                let old_layout = repeat_layout(&self.component_layout, self.capacity)
                    .expect("array layout should be valid");

                unsafe { alloc::realloc(self.data.as_ptr(), old_layout, new_layout.size()) }
            };

            self.data = NonNull::new(new_data).unwrap_or_else(|| handle_alloc_error(new_layout));
        }

        self.capacity = new_capacity;

        self.component_data
            .resize_with(new_capacity, MaybeUninit::uninit);
    }
}

impl Drop for SparseComponentStorage {
    fn drop(&mut self) {
        if self.needs_drop {
            for entity in self.entities.iter() {
                // SAFETY:
                // entity comes from self.entities therefore entity must have been inserted
                // which means entity.index() is a valid index
                let ptr = unsafe { self.get_unchecked(entity.index() as usize) };

                unsafe { (self.drop)(ptr) };
            }
        }

        if self.component_layout.size() > 0 {
            let layout = repeat_layout(&self.component_layout, self.capacity)
                .expect("array layout should be valid");

            unsafe { alloc::dealloc(self.data.as_ptr(), layout) };
        }
    }
}

fn repeat_layout(layout: &Layout, n: usize) -> Option<Layout> {
    let padded_size = layout.pad_to_align().size();
    let repeated_size = padded_size.checked_mul(n)?;

    // SAFETY: layout.align() is already known to be valid and repeated size has just been padded
    unsafe {
        Some(Layout::from_size_align_unchecked(
            repeated_size,
            layout.align(),
        ))
    }
}

#[cfg(test)]
mod tests {}
