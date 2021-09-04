use std::{num::NonZeroU32, path::Path};

use bytemuck::cast_slice;
use glam::UVec2;
use image::io::Reader;
use once_cell::sync::OnceCell;
use wgpu::util::DeviceExt;

use crate::{
    id::{HasId, Id},
    prelude::Color8,
    renderer::RenderCtx,
};

pub struct Texture {
    id: Id<Self>,
    width: u32,
    height: u32,
    pub synced: bool,
    pub buffer: OnceCell<wgpu::Buffer>,
    pub data: OnceCell<Vec<Color8>>,
    pub texture: OnceCell<wgpu::Texture>,
}

impl HasId<Texture> for Texture {
    #[inline]
    fn id(&self) -> Id<Texture> {
        self.id
    }
}

impl Texture {
    #[inline]
    pub fn load(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let image = Reader::open(path.as_ref())?.decode()?;

        let rgba = image.to_rgba8();

        let data: Vec<Color8> = rgba
            .pixels()
            .map(|pixel| Color8::rgba(pixel[0], pixel[1], pixel[2], pixel[3]))
            .collect();

        Ok(Self::from_data(data, rgba.width(), rgba.height()))
    }

    #[inline]
    pub fn from_data(data: Vec<Color8>, width: u32, height: u32) -> Self {
        Self {
            id: Id::new(),
            width,
            height,
            synced: true,
            buffer: OnceCell::new(),
            data: OnceCell::from(data),
            texture: OnceCell::new(),
        }
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[inline]
    pub fn height(&self) -> u32 {
        self.height
    }

    #[inline]
    pub fn size(&self) -> UVec2 {
        UVec2::new(self.width, self.height)
    }

    #[inline]
    pub const fn format(&self) -> wgpu::TextureFormat {
        wgpu::TextureFormat::Rgba8UnormSrgb
    }

    #[inline]
    pub fn sync(&mut self, ctx: &RenderCtx) {
        let texture = if let Some(texture) = self.texture.get() {
            texture
        } else {
            return;
        };

        if self.width == 0 {
            return;
        }

        let bytes_per_row = ((self.width * 4 - 1) / 256 + 1) * 256;
        let size = bytes_per_row as u64 * self.height as u64;

        let buffer = if let Some(buffer) = self.buffer.get() {
            buffer
        } else {
            let buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("texture buffer"),
                size,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });

            self.buffer.set(buffer).unwrap();

            self.buffer.get().unwrap()
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

        if self.data.get().is_none() {
            self.data
                .set(vec![
                    Color8::TRANSPARENT;
                    self.width as usize * self.height as usize
                ])
                .unwrap();
        };

        let mapped = slice.get_mapped_range();
        let data = self.data.get_mut().unwrap();

        for row in 0..self.height {
            let row_offset = bytes_per_row * row;

            for pixel in 0..self.width {
                let pixel_offset = row_offset as usize + pixel as usize * 4;
                let pixel_data = &mapped[pixel_offset..pixel_offset + 4];
                data[row as usize * self.width as usize + pixel as usize] =
                    Color8::rgba(pixel_data[0], pixel_data[1], pixel_data[2], pixel_data[3]);
            }
        }

        buffer.unmap();
    }

    #[inline]
    pub fn data_mut(&mut self) -> &mut Vec<Color8> {
        if self.data.get().is_some() {
            self.data.get_mut().unwrap()
        } else {
            self.data
                .set(vec![
                    Color8::TRANSPARENT;
                    self.width as usize * self.height as usize
                ])
                .unwrap();
            self.data.get_mut().unwrap()
        }
    }

    #[inline]
    pub fn bytes(&self) -> Option<&[u8]> {
        self.data.get().map(|data| cast_slice(data))
    }

    #[inline]
    pub fn texture(&self, ctx: &RenderCtx) -> &wgpu::Texture {
        if let Some(texture) = self.texture.get() {
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

            self.texture.set(texture).unwrap();

            self.texture.get().unwrap()
        }
    }
}
