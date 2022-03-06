use std::collections::HashMap;

use crate::{Handle, HandleId};

pub trait Asset: Send + Sync + 'static {}

impl<T: Send + Sync + 'static> Asset for T {}

pub struct Assets<T: Asset> {
    assets: HashMap<HandleId, T>,
}

impl<T: Asset> Assets<T> {
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
        }
    }

    pub fn add(&mut self, asset: T) -> Handle<T> {
        let mut id = HandleId::random();

        while self.contains(id) {
            id = HandleId::random();
        }

        self.assets.insert(id, asset);

        Handle::new(id)
    }

    pub fn insert(&mut self, handle: impl Into<HandleId>, asset: T) {
        self.assets.insert(handle.into(), asset);
    }

    pub fn contains(&self, handle: impl Into<HandleId>) -> bool {
        self.assets.contains_key(&handle.into())
    }

    pub fn get(&self, handle: impl Into<HandleId>) -> Option<&T> {
        self.assets.get(&handle.into())
    }

    pub fn get_mut(&mut self, handle: impl Into<HandleId>) -> Option<&mut T> {
        self.assets.get_mut(&handle.into())
    }
}

impl<T: Asset> Default for Assets<T> {
    fn default() -> Self {
        Self {
            assets: Default::default(),
        }
    }
}
