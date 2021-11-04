#[cfg(feature = "wgpu")]
use std::sync::atomic::{AtomicPtr, Ordering};

use crate::texture::TextureTrait;

pub(crate) unsafe trait SurfaceTextureTrait: TextureTrait {
    fn present(self: Box<Self>);

    fn texture(&self) -> &crate::Texture;

    fn suboptimal(&self) -> bool;
}

#[cfg(feature = "wgpu")]
struct InnerPtr(AtomicPtr<*const dyn TextureTrait>);

#[cfg(feature = "wgpu")]
struct Inner {
    surface: wgpu::SurfaceTexture,
    ptr: InnerPtr,
}

#[cfg(feature = "wgpu")]
impl Drop for InnerPtr {
    #[inline]
    fn drop(&mut self) {
        let ptr = self.0.load(Ordering::Acquire);

        unsafe { Box::from_raw(ptr) };
    }
}

#[cfg(feature = "wgpu")]
impl std::fmt::Debug for Inner {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.surface.fmt(f)
    }
}

#[cfg(feature = "wgpu")]
unsafe impl TextureTrait for Inner {
    #[inline]
    fn create_view(&self, desc: &crate::TextureViewDescriptor) -> crate::TextureView {
        TextureTrait::create_view(&self.surface.texture, desc)
    }
}

#[cfg(feature = "wgpu")]
unsafe impl SurfaceTextureTrait for Inner {
    #[inline]
    fn present(self: Box<Self>) {
        self.surface.present()
    }

    #[inline]
    fn texture(&self) -> &crate::Texture {
        let ptr = self.ptr.0.load(Ordering::Acquire);

        unsafe { *ptr = &self.surface.texture as *const dyn TextureTrait };

        unsafe { std::mem::transmute(&*ptr) }
    }

    #[inline]
    fn suboptimal(&self) -> bool {
        self.surface.suboptimal
    }
}

pub struct SurfaceTexture(pub(crate) Box<dyn SurfaceTextureTrait>);

impl SurfaceTexture {
    #[inline]
    pub fn create_view(&self, desc: &crate::TextureViewDescriptor) -> crate::TextureView {
        self.0.create_view(desc)
    }

    #[inline]
    pub fn present(self) {
        self.0.present()
    }

    #[inline]
    pub fn texture(&self) -> &crate::Texture {
        self.0.texture()
    }

    #[inline]
    pub fn suboptimal(&self) -> bool {
        self.0.suboptimal()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SurfaceError {
    Timeout,
    Outdated,
    Lost,
    OutOfMemory,
}

pub(crate) unsafe trait SurfaceTrait: Send + Sync {
    fn configure(&self, device: &crate::Device, config: &crate::SurfaceConfiguration);

    fn get_current_texture(&self) -> Result<SurfaceTexture, SurfaceError>;
}

#[cfg(feature = "wgpu")]
unsafe impl SurfaceTrait for wgpu::Surface {
    #[inline]
    fn configure(&self, device: &crate::Device, config: &crate::SurfaceConfiguration) {
        let device = unsafe { &*(device.0.as_ref() as *const _ as *const wgpu::Device) };

        self.configure(device, config);
    }

    fn get_current_texture(&self) -> Result<SurfaceTexture, SurfaceError> {
        let texture = self.get_current_texture();

        match texture {
            Ok(texture) => Ok(SurfaceTexture(Box::new(Inner {
                surface: texture,
                ptr: InnerPtr(AtomicPtr::new(Box::into_raw(Box::new(
                    std::ptr::null::<wgpu::Texture>() as *const _,
                )))),
            }))),
            Err(err) => match err {
                wgpu::SurfaceError::Timeout => Err(SurfaceError::Timeout),
                wgpu::SurfaceError::Outdated => Err(SurfaceError::Outdated),
                wgpu::SurfaceError::Lost => Err(SurfaceError::Lost),
                wgpu::SurfaceError::OutOfMemory => Err(SurfaceError::OutOfMemory),
            },
        }
    }
}

pub struct Surface(pub(crate) Box<dyn SurfaceTrait>);

impl Surface {
    #[cfg(feature = "wgpu")]
    #[inline]
    pub fn new(surface: wgpu::Surface) -> Self {
        Self(Box::new(surface))
    }

    #[inline]
    pub fn configure(&self, device: &crate::Device, config: &crate::SurfaceConfiguration) {
        self.0.configure(device, config)
    }

    #[inline]
    pub fn get_current_texture(&self) -> Result<SurfaceTexture, SurfaceError> {
        self.0.get_current_texture()
    }
}
