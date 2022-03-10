use std::{
    borrow::Cow,
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use ike_task::TaskPool;

use crate::{
    Asset, AssetChannel, AssetIo, AssetIoError, AssetLoader, Handle, HandleId, HandleUntyped,
    LoadContext,
};

pub(crate) struct AssetServerInternal {
    pub(crate) asset_io: Box<dyn AssetIo>,
    pub(crate) channel: RwLock<AssetChannel>,
    pub(crate) loaders: RwLock<Vec<Arc<dyn AssetLoader>>>,
    pub(crate) ext_to_loader_idx: RwLock<HashMap<String, usize>>,
    pub(crate) task_pool: TaskPool,
}

#[derive(Clone)]
pub struct AssetServer {
    pub(crate) inner: Arc<AssetServerInternal>,
}

impl AssetServer {
    pub fn new(asset_io: impl AssetIo, task_pool: TaskPool) -> Self {
        Self::with_boxed_io(Box::new(asset_io), task_pool)
    }

    pub fn with_boxed_io(asset_io: Box<dyn AssetIo>, task_pool: TaskPool) -> Self {
        Self {
            inner: Arc::new(AssetServerInternal {
                asset_io,
                channel: Default::default(),
                loaders: Default::default(),
                ext_to_loader_idx: Default::default(),
                task_pool,
            }),
        }
    }

    pub fn add_loader(&self, loader: impl AssetLoader) {
        let mut loaders = self.inner.loaders.write().unwrap();
        let index = loaders.len();

        for extension in loader.extensions() {
            let is_overlapping = self
                .inner
                .ext_to_loader_idx
                .write()
                .unwrap()
                .insert(extension.to_string(), index)
                .is_some();

            if is_overlapping {
                eprintln!("extensions are overlapping: {}", extension);
            }
        }

        loaders.push(Arc::new(loader));
    }

    fn get_path_asset_loader(
        &self,
        path: &Path,
    ) -> Result<(String, Arc<dyn AssetLoader>), AssetServerError> {
        let file_name = path
            .file_name()
            .ok_or_else(|| AssetServerError::InvalidPath(path.to_path_buf()))?
            .to_str()
            .ok_or_else(|| AssetServerError::InvalidPath(path.to_path_buf()))?
            .to_lowercase();

        let mut ext = file_name.as_str();
        while let Some(idx) = ext.find('.') {
            ext = &ext[idx + 1..];

            if let Ok(loader) = self.get_asset_loader(ext) {
                return Ok((ext.to_string(), loader));
            }
        }

        Err(AssetServerError::LoaderNotFound(file_name))
    }

    fn get_asset_loader(&self, ext: &str) -> Result<Arc<dyn AssetLoader>, AssetServerError> {
        let index = self
            .inner
            .ext_to_loader_idx
            .read()
            .unwrap()
            .get(ext)
            .cloned()
            .ok_or_else(|| AssetServerError::LoaderNotFound(ext.to_string()))?;

        let loader = self.inner.loaders.read().unwrap()[index].clone();

        Ok(loader)
    }

    async fn load_async(&self, path: &Path) -> Result<(), AssetServerError> {
        let (ext, loader) = self.get_path_asset_loader(path)?;

        let bytes = self.inner.asset_io.load_path(path).await?;

        let mut load_context = LoadContext {
            bytes: &bytes,
            default_id: HandleId::from(path),
            path: Some(path),
            extension: &ext,
            asset_server: self,
        };

        loader.load(&mut load_context).await?;

        Ok(())
    }

    async fn load_from_bytes_async<'a>(
        &self,
        bytes: Cow<'a, [u8]>,
        ext: &str,
        id: HandleId,
    ) -> Result<(), AssetServerError> {
        let loader = self.get_asset_loader(ext)?;

        let mut load_context = LoadContext {
            bytes: &bytes,
            default_id: id,
            path: None,
            extension: &ext,
            asset_server: self,
        };

        loader.load(&mut load_context).await?;

        Ok(())
    }

    fn load_untracked(&self, path: &Path) -> HandleId {
        let server = self.clone();
        let owned_path = path.to_path_buf();
        self.inner
            .task_pool
            .spawn(async move {
                if let Err(err) = server.load_async(&owned_path).await {
                    eprintln!("{}", err);
                }
            })
            .detach();

        let handle_id = HandleId::from(path);

        handle_id
    }

    fn load_untracked_from_bytes(&self, bytes: Cow<'static, [u8]>, ext: &str) -> HandleId {
        let handle_id = HandleId::random();
        let server = self.clone();
        let owned_ext = ext.to_owned();
        self.inner
            .task_pool
            .spawn(async move {
                if let Err(err) = server
                    .load_from_bytes_async(bytes, &owned_ext, handle_id)
                    .await
                {
                    eprintln!("{}", err);
                }
            })
            .detach();

        handle_id
    }

    pub fn load_untyped(&self, path: impl AsRef<Path>) -> HandleUntyped {
        let handle_id = self.load_untracked(path.as_ref());
        HandleUntyped::new_weak(handle_id)
    }

    pub fn load_untyped_from_bytes(
        &self,
        bytes: impl Into<Cow<'static, [u8]>>,
        ext: impl AsRef<str>,
    ) -> HandleUntyped {
        let handle_id = self.load_untracked_from_bytes(bytes.into(), ext.as_ref());
        HandleUntyped::new_weak(handle_id)
    }

    pub fn load<T: Asset>(&self, path: impl AsRef<Path>) -> Handle<T> {
        self.load_untyped(path).typed()
    }

    pub fn load_from_bytes<T: Asset>(
        &self,
        bytes: impl Into<Cow<'static, [u8]>>,
        ext: impl AsRef<str>,
    ) -> Handle<T> {
        self.load_untyped_from_bytes(bytes, ext).typed()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum AssetServerError {
    #[error("loader not found for: {0}")]
    LoaderNotFound(String),
    #[error("path is invalid: {0}")]
    InvalidPath(PathBuf),
    #[error("asset loader error '{0}'")]
    LoaderError(#[from] ike_util::Error),
    #[error("asset io error '{0}'")]
    AssetIoError(#[from] AssetIoError),
}
