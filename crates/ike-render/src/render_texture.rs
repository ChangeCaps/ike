use glam::UVec2;

use ike_wgpu as wgpu;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RenderTarget {
    pub samples: u32,
    pub format: wgpu::TextureFormat,
}

pub struct RenderTexture {
    view: wgpu::TextureView,
    size: UVec2,
    recreate: bool,
    samples: u32,
    format: wgpu::TextureFormat,
}

impl RenderTexture {
    #[inline]
    pub fn new(
        view: wgpu::TextureView,
        size: UVec2,
        samples: u32,
        format: wgpu::TextureFormat,
    ) -> Self {
        Self {
            view,
            size,
            recreate: true,
            samples,
            format,
        }
    }

    #[inline]
    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    #[inline]
    pub fn size(&self) -> UVec2 {
        self.size
    }

    #[inline]
    pub fn samples(&self) -> u32 {
        self.samples
    }

    #[inline]
    pub fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    #[inline]
    pub fn target(&self) -> RenderTarget {
        RenderTarget {
            samples: self.samples,
            format: self.format,
        }
    }
}
