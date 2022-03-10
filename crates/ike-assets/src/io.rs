use std::path::{Path, PathBuf};

use ike_util::BoxedFuture;

#[derive(thiserror::Error, Debug)]
pub enum AssetIoError {
    #[error("invalid path: {0}")]
    NotFound(PathBuf),
}

pub trait AssetIo: Send + Sync + 'static {
    fn load_path<'a>(&'a self, path: &'a Path) -> BoxedFuture<'a, Result<Vec<u8>, AssetIoError>>;
    fn is_directory(&self, path: &Path) -> bool;
}
