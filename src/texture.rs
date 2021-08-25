use std::{io::Read, path::Path};

use image::io::Reader;
use wgpu::util::DeviceExt;

use crate::{id::Id, renderer::RenderCtx};

pub struct Texture {
    pub id: Id,
    pub width: u32,
    pub height: u32,
    pub data: Option<Vec<u8>>,
    pub texture: Option<wgpu::Texture>,
}

impl Texture {
    #[inline]
    pub fn load(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let image = Reader::open(path.as_ref())?.decode()?;

        let rgba = image.to_rgba8();

        Ok(Self {
            id: Id::new(),
            width: rgba.width(),
            height: rgba.height(),
            data: Some(rgba.bytes().try_fold(
                Vec::with_capacity(rgba.len()),
                |mut acc, val| -> anyhow::Result<Vec<u8>> {
                    acc.push(val?);
                    Ok(acc)
                },
            )?),
            texture: None,
        })
    }

    #[inline]
    pub fn texture(&mut self, ctx: &RenderCtx) -> &wgpu::Texture {
        if let Some(ref texture) = self.texture {
            texture
        } else {
            let texture = ctx.device.create_texture_with_data(
                &ctx.queue,
                &wgpu::TextureDescriptor {
                    label: Some("texture"),
                    size: wgpu::Extent3d {
                        width: self.width,
                        height: self.height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
                },
                self.data.as_ref().unwrap(),
            );

            self.texture = Some(texture);

            self.texture.as_ref().unwrap()
        }
    }
}
