use std::{num::NonZeroU32, path::Path};

use image::io::Reader;
use wgpu::util::DeviceExt;

use crate::{id::Id, prelude::Color, renderer::RenderCtx};

pub struct Texture {
    pub id: Id,
    pub width: u32,
    pub height: u32,
    pub synced: bool,
    pub buffer: Option<wgpu::Buffer>,
    pub data: Option<Vec<Color>>,
    pub texture: Option<wgpu::Texture>,
}

impl Texture {
    #[inline]
    pub fn load(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let image = Reader::open(path.as_ref())?.decode()?;

        let rgba = image.to_rgba8();

        let data = rgba
            .pixels()
            .map(|pixel| {
                Color::rgba(
                    pixel[0] as f32 / 255.0,
                    pixel[1] as f32 / 255.0,
                    pixel[2] as f32 / 255.0,
                    pixel[3] as f32 / 255.0,
                )
            })
            .collect();

        Ok(Self {
            id: Id::new(),
            width: rgba.width(),
            height: rgba.height(),
            synced: true,
            buffer: None,
            data: Some(data),
            texture: None,
        })
    }

    #[inline]
    pub const fn format(&self) -> wgpu::TextureFormat {
        wgpu::TextureFormat::Rgba8UnormSrgb
    }

    #[inline]
    pub fn sync(&mut self, ctx: &RenderCtx) {
        let texture = if let Some(ref texture) = self.texture {
            texture
        } else {
            return;
        };

        if self.width == 0 {
            return;
        }

        let bytes_per_row = ((self.width * 4 - 1) / 256 + 1) * 256;
        let size = bytes_per_row as u64 * self.height as u64;

        let buffer = if let Some(ref buffer) = self.buffer {
            buffer
        } else {
            let buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("texture buffer"),
                size,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });

            self.buffer = Some(buffer);

            self.buffer.as_ref().unwrap()
        };

        let mut encoder = ctx.device.create_command_encoder(&Default::default());

        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTextureBase {
                texture,
                mip_level: 0,
                origin: Default::default(),
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBufferBase {
                buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(unsafe { NonZeroU32::new_unchecked(bytes_per_row) }),
                    rows_per_image: None,
                },
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );

        ctx.queue.submit(std::iter::once(encoder.finish()));

        let slice = buffer.slice(..);
        let fut = slice.map_async(wgpu::MapMode::Read);

        ctx.device.poll(wgpu::Maintain::Wait);

        pollster::block_on(fut).unwrap();

        if self.data.is_none() {
            self.data = Some(vec![
                Color::TRANSPARENT;
                self.width as usize * self.height as usize
            ]);
        };

        let mapped = slice.get_mapped_range();
        let data = self.data.as_mut().unwrap();

        for row in 0..self.height {
            let row_offset = bytes_per_row * row;

            for pixel in 0..self.width {
                let pixel_offset = row_offset as usize + pixel as usize * 4;
                let pixel_data = &mapped[pixel_offset..pixel_offset + 4];
                data[row as usize * self.width as usize + pixel as usize] =
                    Color::from([pixel_data[0], pixel_data[1], pixel_data[2], pixel_data[3]]);
            }
        }

        buffer.unmap();
    }

    #[inline]
    pub fn bytes(&self) -> Option<Vec<u8>> {
        self.data.as_ref().map(|data| {
            data.iter()
                .flat_map(|pixel| <Color as Into<[u8; 4]>>::into(*pixel))
                .collect()
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
                &self.bytes().unwrap(),
            );

            self.texture = Some(texture);

            self.texture.as_ref().unwrap()
        }
    }
}
