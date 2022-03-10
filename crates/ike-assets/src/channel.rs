use std::{
    any::{Any, TypeId},
    collections::HashMap,
    mem,
};

use crate::{Asset, HandleId};

#[derive(Default)]
pub struct AssetChannel {
    assets: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl AssetChannel {
    pub fn get_assets<T: Asset>(&self) -> Option<&Vec<(HandleId, T)>> {
        self.assets.get(&TypeId::of::<T>())?.downcast_ref()
    }

    pub fn push<T: Asset>(&mut self, handle: impl Into<HandleId>, asset: T) {
        let assets = self
            .assets
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(Vec::<(HandleId, T)>::new()))
            .downcast_mut::<Vec<(HandleId, T)>>()
            .unwrap();

        assets.push((handle.into(), asset));
    }

    pub fn remove_assets<T: Asset>(&mut self) -> impl Iterator<Item = (HandleId, T)> {
        if let Some(assets) = self.assets.get_mut(&TypeId::of::<T>()) {
            mem::take(assets.downcast_mut::<Vec<(HandleId, T)>>().unwrap()).into_iter()
        } else {
            Vec::new().into_iter()
        }
    }
}
