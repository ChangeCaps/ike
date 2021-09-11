use std::{num::NonZeroU32, path::Path};

use bytemuck::cast_slice;
use glam::UVec2;
use image::io::Reader;
use once_cell::sync::OnceCell;

use crate::{
    id::{HasId, Id},
    prelude::Color8,
    renderer::RenderCtx,
};

pub struct Texture {
    id: Id<Self>,
    width: u32,
    height: u32,
    synced: bool,
    buffer: OnceCell<ike_wgpu::Buffer>,
    data: OnceCell<Vec<Color8>>,
    texture: OnceCell<ike_wgpu::Texture>,
}

impl Default for Texture {
    #[inline]
    fn default() -> Self {
        Self {
            id: Id::new(),
            width: 1,
            height: 1,
            synced: true,
            buffer: Default::default(),
            data: Default::default(),
            texture: Default::default(),
        }
    }
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
    pub const fn format(&self) -> ike_wgpu::TextureFormat {
        ike_wgpu::TextureFormat::Rgba8UnormSrgb
    }

    #[inline]
    pub fn sync_down(&mut self, ctx: &RenderCtx) {
        self.synced = true;

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
            let buffer = ctx.device.create_buffer(&ike_wgpu::BufferDescriptor {
                label: Some("texture buffer"),
                size,
                usage: ike_wgpu::BufferUsages::COPY_DST | ike_wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });

            let _ = self.buffer.set(buffer);

            self.buffer.get().unwrap()
        };

        let mut encoder = ctx.device.create_command_encoder(&Default::default());

        encoder.copy_texture_to_buffer(
            ike_wgpu::ImageCopyTexture {
                texture,
                mip_level: 0,
                origin: Default::default(),
                aspect: ike_wgpu::TextureAspect::All,
            },
            ike_wgpu::ImageCopyBuffer {
                buffer,
                layout: ike_wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: unsafe { Some(NonZeroU32::new_unchecked(bytes_per_row)) },
                    rows_per_image: None,
                },
            },
            ike_wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );

        ctx.queue.submit(std::iter::once(encoder.finish()));

        let slice = buffer.slice(..);
        let fut = slice.map_async(ike_wgpu::MapMode::Read);

        ctx.device.poll(ike_wgpu::Maintain::Wait);

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
    pub fn sync_up(&mut self, ctx: &RenderCtx) {
        if self.synced || self.width == 0 || self.height == 0 {
            return;
        }

        self.synced = true;

        self.data_mut();

        let data = self.bytes().unwrap();

        let texture = self.texture(ctx);

        let bytes_per_row = ((self.width * 4 - 1) / 256 + 1) * 256;

        ctx.queue.write_texture(
            ike_wgpu::ImageCopyTexture {
                texture,
                mip_level: 0,
                origin: ike_wgpu::Origin3d::default(),
                aspect: ike_wgpu::TextureAspect::All,
            },
            data,
            ike_wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: unsafe { Some(NonZeroU32::new_unchecked(bytes_per_row)) },
                rows_per_image: None,
            },
            ike_wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );
    }

    #[inline]
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;

        self.synced = false;
        self.texture.take();
        self.data.take();
        self.buffer.take();
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
    pub fn write(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        image::save_buffer(
            path,
            self.bytes()
                .ok_or_else(|| anyhow::Error::msg("no data in texture"))?,
            self.width,
            self.height,
            image::ColorType::Rgba8,
        )?;

        Ok(())
    }

    #[inline]
    pub fn texture(&self, ctx: &RenderCtx) -> &ike_wgpu::Texture {
        if let Some(texture) = self.texture.get() {
            texture
        } else {
            let texture = if let Some(data) = self.bytes() {
                let texture = ctx.device.create_texture_with_data(
                    &ctx.queue,
                    &ike_wgpu::TextureDescriptor {
                        label: Some("texture"),
                        size: ike_wgpu::Extent3d {
                            width: self.width,
                            height: self.height,
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: ike_wgpu::TextureDimension::D2,
                        format: ike_wgpu::TextureFormat::Rgba8UnormSrgb,
                        usage: ike_wgpu::TextureUsages::COPY_DST
                            | ike_wgpu::TextureUsages::TEXTURE_BINDING,
                    },
                    data,
                );

                texture
            } else {
                let texture = ctx.device.create_texture(&ike_wgpu::TextureDescriptor {
                    label: Some("texture"),
                    size: ike_wgpu::Extent3d {
                        width: self.width,
                        height: self.height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: ike_wgpu::TextureDimension::D2,
                    format: ike_wgpu::TextureFormat::Rgba8UnormSrgb,
                    usage: ike_wgpu::TextureUsages::COPY_DST
                        | ike_wgpu::TextureUsages::TEXTURE_BINDING,
                });

                texture
            };

            self.texture.set(texture).unwrap();

            self.texture.get().unwrap()
        }
    }
}
