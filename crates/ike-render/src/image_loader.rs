use ike_assets::{AssetLoader, LoadContext, LoadedAsset};
use ike_util::BoxedFuture;

use crate::{Extent3d, FilterMode, Image, TextureDimension, TextureFormat, TextureUsages};

const EXTENSIONS: &[&str] = &[
    #[cfg(feature = "png")]
    "png",
    #[cfg(feature = "png")]
    "normal.png",
    #[cfg(feature = "jpg")]
    "jpg",
    #[cfg(feature = "jpg")]
    "normal.jpg",
];

fn is_normal(ext: &str) -> bool {
    match ext {
        "normal.png" => true,
        "normal.jpg" => true,
        _ => false,
    }
}

pub struct ImageLoader;

impl AssetLoader for ImageLoader {
    fn load<'a>(
        &'a self,
        load_context: &'a mut LoadContext<'a>,
    ) -> BoxedFuture<'a, Result<(), ike_util::Error>> {
        Box::pin(async {
            let image = image::load_from_memory(load_context.bytes())?;

            let data = image.to_rgba8().to_vec();

            let format = if is_normal(load_context.extension()) {
                TextureFormat::Rgba8Unorm
            } else {
                TextureFormat::Rgba8UnormSrgb
            };

            let image = Image {
                data,
                format,
                size: Extent3d {
                    width: image.width(),
                    height: image.height(),
                    depth_or_array_layers: 1,
                },
                dimension: TextureDimension::D2,
                mip_level_count: 1,
                sample_count: 1,
                usage: TextureUsages::TEXTURE_BINDING,
                min_filter: FilterMode::Linear,
                mag_filter: FilterMode::Linear,
            };

            load_context.set_default_asset(LoadedAsset::new(image));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        EXTENSIONS
    }
}
