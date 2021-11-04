use glam::UVec2;

use ike_wgpu as wgpu;

enum Inner {
    Surface(wgpu::SurfaceTexture),
    Texture(wgpu::Texture),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RenderTarget {
    pub samples: u32,
    pub format: wgpu::TextureFormat,
}

pub struct RenderTexture {
    inner: Inner,
    pub size: UVec2,
    pub recreate: bool,
    pub samples: u32,
    pub format: wgpu::TextureFormat,
}

impl RenderTexture {
    #[inline]
    pub fn from_surface_texture(
        texture: wgpu::SurfaceTexture,
        size: UVec2,
        samples: u32,
        format: wgpu::TextureFormat,
    ) -> Self {
        Self {
            inner: Inner::Surface(texture),
            size,
            recreate: true,
            samples,
            format,
        }
    }

    #[inline]
    pub fn from_texture(
        texture: wgpu::Texture,
        size: UVec2,
        samples: u32,
        format: wgpu::TextureFormat,
    ) -> Self {
        Self {
            inner: Inner::Texture(texture),
            size,
            recreate: true,
            samples,
            format,
        }
    }

    #[inline]
    pub fn texture(&self) -> &wgpu::Texture {
        match self.inner {
            Inner::Surface(ref texture) => texture.texture(),
            Inner::Texture(ref texture) => texture,
        }
    }

    #[inline]
    pub fn present(self) {
        match self.inner {
            Inner::Surface(texture) => texture.present(),
            Inner::Texture(_) => {}
        }
    }

    #[inline]
    pub fn target(&self) -> RenderTarget {
        RenderTarget {
            samples: self.samples,
            format: self.format,
        }
    }
}
