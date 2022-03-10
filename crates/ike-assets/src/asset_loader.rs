use std::path::Path;

use ike_util::BoxedFuture;

use crate::{Asset, AssetIoError, AssetServer, Handle, HandleId};

pub trait AssetLoader: Send + Sync + 'static {
    fn load<'a>(
        &'a self,
        load_context: &'a mut LoadContext<'a>,
    ) -> BoxedFuture<'a, Result<(), ike_util::Error>>;

    fn extensions(&self) -> &[&str];
}

pub struct LoadedAsset<T> {
    asset: T,
    dependencies: Vec<HandleId>,
}

impl<T> LoadedAsset<T> {
    pub fn new(asset: T) -> Self {
        Self {
            asset,
            dependencies: Vec::new(),
        }
    }

    pub fn add_dependency(&mut self, id: impl Into<HandleId>) {
        self.dependencies.push(id.into());
    }

    #[must_use]
    pub fn with_dependency(mut self, id: impl Into<HandleId>) -> Self {
        self.add_dependency(id);
        self
    }
}

pub struct LoadContext<'a> {
    pub(crate) bytes: &'a [u8],
    pub(crate) default_id: HandleId,
    pub(crate) path: Option<&'a Path>,
    pub(crate) extension: &'a str,
    pub(crate) asset_server: &'a AssetServer,
}

impl<'a> LoadContext<'a> {
    pub fn bytes(&self) -> &'a [u8] {
        self.bytes
    }

    pub fn path(&self) -> Option<&'a Path> {
        self.path
    }

    pub fn extension(&self) -> &'a str {
        self.extension
    }

    pub async fn read_path(&self, path: impl AsRef<Path>) -> Result<Vec<u8>, AssetIoError> {
        self.asset_server
            .inner
            .asset_io
            .load_path(path.as_ref())
            .await
    }

    pub fn load<T: Asset>(&self, path: impl AsRef<Path>) -> Handle<T> {
        self.asset_server.load(path)
    }

    pub fn set_asset<T: Asset>(
        &mut self,
        id: impl Into<HandleId>,
        asset: LoadedAsset<T>,
    ) -> Handle<T> {
        let id = id.into();
        self.asset_server
            .inner
            .channel
            .write()
            .unwrap()
            .push(id.clone(), asset.asset);
        Handle::new_weak(id)
    }

    pub fn set_default_asset<T: Asset>(&mut self, asset: LoadedAsset<T>) {
        self.set_asset(self.default_id, asset);
    }

    pub fn add_asset<T: Asset>(&mut self, asset: LoadedAsset<T>) -> Handle<T> {
        self.set_asset(HandleId::random(), asset)
    }
}
