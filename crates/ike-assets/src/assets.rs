use std::collections::HashMap;

use crate::{AssetEvent, Handle, HandleId};

pub trait Asset: Send + Sync + 'static {}

impl<T: Send + Sync + 'static> Asset for T {}

pub struct Assets<T: Asset> {
    assets: HashMap<HandleId, T>,
    pub(crate) events: Vec<AssetEvent<T>>,
}

impl<T: Asset> Assets<T> {
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
            events: Vec::new(),
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
        let handle = handle.into();

        self.events.push(AssetEvent::Created {
            handle: Handle::new_weak(handle),
        });

        self.assets.insert(handle, asset);
    }

    pub fn remove(&mut self, handle: impl Into<HandleId>) -> Option<T> {
        let handle = handle.into();

        let removed = self.assets.remove(&handle);

        if removed.is_some() {
            self.events.push(AssetEvent::Removed {
                handle: Handle::new_weak(handle),
            });
        }

        removed
    }

    pub fn contains(&self, handle: impl Into<HandleId>) -> bool {
        self.assets.contains_key(&handle.into())
    }

    pub fn get(&self, handle: impl Into<HandleId>) -> Option<&T> {
        self.assets.get(&handle.into())
    }

    pub fn get_mut(&mut self, handle: impl Into<HandleId>) -> Option<&mut T> {
        let handle = handle.into();

        self.events.push(AssetEvent::Modified {
            handle: Handle::new_weak(handle),
        });

        self.assets.get_mut(&handle)
    }
}

impl<T: Asset> Default for Assets<T> {
    fn default() -> Self {
        Self::new()
    }
}
