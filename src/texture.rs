use std::{fmt::Debug, io::BufReader, num::NonZeroU32, path::Path};

use bytemuck::cast_slice;
use glam::UVec2;
use image::{hdr::HdrDecoder, io::Reader};
use once_cell::sync::OnceCell;

use crate::{
    id::{HasId, Id},
    prelude::{Color, Color8},
    renderer::RenderCtx,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TextureVersion(u64);

impl Default for TextureVersion {
    #[inline]
    fn default() -> Self {
        Self(0)
    }
}

pub trait TextureFormat: Clone + Default {
    type Data: bytemuck::Pod + bytemuck::Zeroable;

    fn format(&self) -> ike_wgpu::TextureFormat;
}

#[derive(Clone)]
pub struct Rgba8Unorm {
    pub color_space: ColorSpace,
}

impl Default for Rgba8Unorm {
    #[inline]
    fn default() -> Self {
        Self {
            color_space: ColorSpace::Gamma,
        }
    }
}

impl TextureFormat for Rgba8Unorm {
    type Data = Color8;

    #[inline]
    fn format(&self) -> ike_wgpu::TextureFormat {
        self.color_space.format()
    }
}

#[derive(Clone, Default)]
pub struct Rgba32Float;

impl TextureFormat for Rgba32Float {
    type Data = Color;

    #[inline]
    fn format(&self) -> ike_wgpu::TextureFormat {
        ike_wgpu::TextureFormat::Rgba32Float
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ColorSpace {
    Linear,
    Gamma,
}

impl ColorSpace {
    fn format(&self) -> ike_wgpu::TextureFormat {
        match self {
            Self::Linear => ike_wgpu::TextureFormat::Rgba8Unorm,
            Self::Gamma => ike_wgpu::TextureFormat::Rgba8UnormSrgb,
        }
    }
}

pub struct Texture<F: TextureFormat = Rgba8Unorm> {
    id: Id<Self>,
    version: u64,
    local_version: u64,
    width: u32,
    height: u32,
    format: F,
    buffer: OnceCell<ike_wgpu::Buffer>,
    data: OnceCell<Vec<F::Data>>,
    texture: OnceCell<ike_wgpu::Texture>,
}

pub type HdrTexture = Texture<Rgba32Float>;

impl<F: TextureFormat> Default for Texture<F> {
    #[inline]
    fn default() -> Self {
        Self {
            id: Id::new(),
            version: 1,
            local_version: 0,
            width: 1,
            height: 1,
            format: F::default(),
            buffer: Default::default(),
            data: Default::default(),
            texture: Default::default(),
        }
    }
}

impl<F: TextureFormat> HasId<Texture<F>> for Texture<F> {
    #[inline]
    fn id(&self) -> Id<Texture<F>> {
        self.id
    }
}

impl<F: TextureFormat> Clone for Texture<F> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            id: Id::new(),
            version: 1,
            local_version: 0,
            width: self.width,
            height: self.height,
            format: self.format.clone(),
            buffer: OnceCell::new(),
            data: self.data.clone(),
            texture: OnceCell::new(),
        }
    }
}

impl<F: TextureFormat> std::fmt::Debug for Texture<F> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Texture")
            .field("id", &self.id)
            .field("version", &self.version)
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}

impl HdrTexture {
    #[inline]
    pub fn load(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let file = std::fs::File::open(path.as_ref())?;
        let buf_reader = BufReader::new(file);

        let image = HdrDecoder::new(buf_reader)?;

        let meta = image.metadata();

        let rgba = image.read_image_hdr()?;

        let data: Vec<Color> = rgba
            .into_iter()
            .map(|pixel| Color::rgba(pixel[0], pixel[1], pixel[2], 1.0))
            .collect();

        Ok(Self::from_data(data, meta.width, meta.height))
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
    pub fn color_space(&self) -> ColorSpace {
        self.format.color_space
    }

    #[inline]
    pub fn set_color_space(&mut self, color_space: ColorSpace) {
        self.version += 1;
        self.format.color_space = color_space;
        self.texture.take();
    }
}

impl<F: TextureFormat> Texture<F> {
    #[inline]
    pub fn from_size(size: UVec2) -> Self {
        Self {
            width: size.x,
            height: size.y,
            ..Default::default()
        }
    }

    #[inline]
    pub fn from_data(data: Vec<F::Data>, width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: OnceCell::from(data),
            ..Default::default()
        }
    }

    #[inline]
    pub fn version(&self) -> TextureVersion {
        TextureVersion(self.version)
    }

    #[inline]
    pub fn outdated(&self, version: TextureVersion) -> bool {
        self.version > version.0
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
    pub fn resize(&mut self, width: u32, height: u32) {
        if self.width != width || self.height != height {
            self.version += 1;
            self.width = width;
            self.height = height;

            self.texture.take();
            self.data.take();
            self.buffer.take();
        }
    }

    #[inline]
    pub fn format(&self) -> ike_wgpu::TextureFormat {
        self.format.format()
    }

    #[inline]
    pub fn sync_down(&mut self, ctx: &RenderCtx) {
        self.local_version = self.version;

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
            let _ = self.data.set(vec![
                bytemuck::Zeroable::zeroed();
                self.width as usize * self.height as usize
            ]);
        };

        let mapped = slice.get_mapped_range();
        let data = self.data.get_mut().unwrap();

        for row in 0..self.height {
            let row_offset = bytes_per_row * row;

            for pixel in 0..self.width {
                let pixel_offset = row_offset as usize + pixel as usize * 4;
                let pixel_data = &mapped[pixel_offset..pixel_offset + 4];
                data[row as usize * self.width as usize + pixel as usize] =
                    cast_slice(pixel_data)[0];
            }
        }

        buffer.unmap();
    }

    #[inline]
    pub fn sync_up(&mut self, ctx: &RenderCtx) {
        if self.local_version == self.version || self.width == 0 || self.height == 0 {
            return;
        }

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
    pub fn data_mut(&mut self) -> &mut Vec<F::Data> {
        self.version += 1;

        if self.data.get().is_some() {
            self.data.get_mut().unwrap()
        } else {
            let _ = self.data.set(vec![
                bytemuck::Zeroable::zeroed();
                self.width as usize * self.height as usize
            ]);
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
                        format: self.format(),
                        usage: ike_wgpu::TextureUsages::COPY_DST
                            | ike_wgpu::TextureUsages::TEXTURE_BINDING
                            | ike_wgpu::TextureUsages::RENDER_ATTACHMENT,
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
                    format: self.format(),
                    usage: ike_wgpu::TextureUsages::COPY_DST
                        | ike_wgpu::TextureUsages::TEXTURE_BINDING
                        | ike_wgpu::TextureUsages::RENDER_ATTACHMENT,
                });

                texture
            };

            self.texture.set(texture).unwrap();

            self.texture.get().unwrap()
        }
    }
}
