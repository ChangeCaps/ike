use ike_assets::{AssetEvent, Assets};
use ike_ecs::{EventReader, Res, ResMut};

use crate::{
    Extent3d, RenderDevice, RenderQueue, Sampler, SamplerDescriptor, Texture, TextureDescriptor,
    TextureDimension, TextureFormat, TextureUsages, TextureView,
};

fn pixel_size(format: TextureFormat) -> usize {
    let info = format.describe();

    info.block_size as usize * info.block_dimensions.0 as usize * info.block_dimensions.1 as usize
}

pub struct Image {
    pub data: Vec<u8>,
    pub format: TextureFormat,
    pub size: Extent3d,
    pub dimension: TextureDimension,
    pub mip_level_count: u32,
    pub sample_count: u32,
    pub usage: TextureUsages,
}

impl Default for Image {
    fn default() -> Self {
        let format = TextureFormat::Rgba8UnormSrgb;
        let pixel_size = pixel_size(format);

        Self {
            data: vec![u8::MAX; pixel_size],
            format,
            size: Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            dimension: TextureDimension::D2,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
        }
    }
}

impl Image {
    pub fn new_2d(data: impl Into<Vec<u8>>, width: u32, height: u32) -> Self {
        Self {
            data: data.into(),
            format: TextureFormat::Rgba8UnormSrgb,
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            dimension: TextureDimension::D2,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING,
        }
    }

    pub fn create_texture(&self, device: &RenderDevice, queue: &RenderQueue) -> ImageTexture {
        let texture = device.create_texture_with_data(
            queue,
            &TextureDescriptor {
                label: Some("ike_image_texture"),
                size: self.size,
                format: self.format,
                dimension: self.dimension,
                mip_level_count: self.mip_level_count,
                sample_count: self.sample_count,
                usage: self.usage,
            },
            &self.data,
        );

        let texture_view = texture.create_view(&Default::default());

        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("ike_image_sampler"),
            ..Default::default()
        });

        ImageTexture {
            texture,
            texture_view,
            sampler,
            size: self.size,
        }
    }
}

pub fn image_texture_system(
    mut event_reader: EventReader<AssetEvent<Image>>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    images: Res<Assets<Image>>,
    mut image_textures: ResMut<Assets<ImageTexture>>,
) {
    for asset_event in event_reader.iter() {
        match asset_event {
            AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                if let Some(image) = images.get(handle) {
                    if let Some(image_texture) = image_textures.get_mut(handle) {
                        *image_texture = image.create_texture(&render_device, &render_queue);
                    } else {
                        image_textures
                            .insert(handle, image.create_texture(&render_device, &render_queue));
                    }
                }
            }
            AssetEvent::Removed { handle } => {
                image_textures.remove(handle);
            }
        }
    }
}

pub struct ImageTexture {
    pub texture: Texture,
    pub texture_view: TextureView,
    pub sampler: Sampler,
    pub size: Extent3d,
}
