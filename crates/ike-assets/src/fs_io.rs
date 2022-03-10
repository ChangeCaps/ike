use std::path::Path;

use async_fs::File;
use futures_lite::AsyncReadExt;
use ike_util::BoxedFuture;

use crate::{AssetIo, AssetIoError};

pub struct FsIo;

impl AssetIo for FsIo {
    fn load_path<'a>(&'a self, path: &'a Path) -> BoxedFuture<'a, Result<Vec<u8>, AssetIoError>> {
        Box::pin(async move {
            let mut bytes = Vec::new();

            match File::open(path).await {
                Ok(mut file) => {
                    file.read_to_end(&mut bytes).await.unwrap();

                    Ok(bytes)
                }
                Err(_) => Err(AssetIoError::NotFound(path.to_path_buf())),
            }
        })
    }

    fn is_directory(&self, path: &Path) -> bool {
        path.is_dir()
    }
}
