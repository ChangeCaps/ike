use std::collections::HashMap;

use crate::Handle;

pub struct Assets<T: 'static> {
    inner: HashMap<Handle<T>, T>,
    next_id: u64,
}

impl<T> Default for Assets<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Assets<T> {
    #[inline]
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
            next_id: 0,
        }
    }

    #[inline]
    pub fn add(&mut self, asset: T) -> Handle<T> {
        let handle = Handle::new(self.next_id);
        self.next_id += 1;

        self.insert(handle.clone(), asset);

        handle
    }

    #[inline]
    pub fn insert(&mut self, handle: Handle<T>, asset: T) {
        self.inner.insert(handle, asset);
    }

    #[inline]
    pub fn get(&self, handle: &Handle<T>) -> Option<&T> {
        self.inner.get(handle)
    }

    #[inline]
    pub fn get_mut(&mut self, handle: &Handle<T>) -> Option<&mut T> {
        self.inner.get_mut(handle)
    }

    #[inline]
    pub fn clean(&mut self) {
        self.inner.retain(|handle, _| handle.shared());
    }
}
