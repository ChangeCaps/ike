use std::{
    any::{type_name, Any},
    collections::HashMap,
};

#[derive(Default)]
pub struct EditorData {
    data: HashMap<&'static str, Box<dyn Any + Send + Sync>>,
}

impl EditorData {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn insert<T: Any + Send + Sync>(&mut self, data: T) {
        self.data.insert(type_name::<T>(), Box::new(data));
    }

    #[inline]
    pub fn remove<T: Any + Send + Sync>(&mut self) -> Option<Box<T>> {
        if let Some(data) = self.data.remove(type_name::<T>()) {
            Some(unsafe { Box::from_raw(Box::into_raw(data) as *mut _) })
        } else {
            None
        }
    }

    #[inline]
    pub fn get_or_insert_with<T: Any + Send + Sync>(&mut self, f: impl FnOnce() -> T) -> &mut T {
        if !self.contains::<T>() {
            self.insert(f());
        }

        self.get_mut().unwrap()
    }

    #[inline]
    pub fn contains<T: Any + Send + Sync>(&self) -> bool {
        self.data.contains_key(type_name::<T>())
    }

    #[inline]
    pub fn get<T: Any + Send + Sync>(&self) -> Option<&T> {
        if let Some(data) = self.data.get(type_name::<T>()) {
            Some(unsafe { &*(data.as_ref() as *const _ as *const _) })
        } else {
            None
        }
    }

    #[inline]
    pub fn get_mut<T: Any + Send + Sync>(&mut self) -> Option<&mut T> {
        if let Some(data) = self.data.get_mut(type_name::<T>()) {
            Some(unsafe { &mut *(data.as_mut() as *mut _ as *mut _) })
        } else {
            None
        }
    }
}
