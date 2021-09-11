use crate::texture::TextureTrait;

pub(crate) unsafe trait SurfaceTextureTrait: TextureTrait {}

#[cfg(feature = "wgpu")]
unsafe impl TextureTrait for wgpu::SurfaceTexture {
    #[inline]
    fn create_view(&self, desc: &crate::TextureViewDescriptor) -> crate::TextureView {
        TextureTrait::create_view(&self.texture, desc)
    }
}

#[cfg(feature = "wgpu")]
unsafe impl SurfaceTextureTrait for wgpu::SurfaceTexture {}

pub struct SurfaceTexture(pub(crate) Box<dyn SurfaceTextureTrait>);

impl SurfaceTexture {
    #[inline]
    pub fn create_view(&self, desc: &crate::TextureViewDescriptor) -> crate::TextureView {
        self.0.create_view(desc)
    }
}

pub struct SurfaceFrame {
    pub output: SurfaceTexture,
    pub suboptimal: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SurfaceError {
    Timeout,
    Outdated,
    Lost,
    OutOfMemory,
}

pub(crate) unsafe trait SurfaceTrait {
    fn configure(&self, device: &crate::Device, config: &crate::SurfaceConfiguration);

    fn get_current_frame(&self) -> Result<SurfaceFrame, SurfaceError>;
}

#[cfg(feature = "wgpu")]
unsafe impl SurfaceTrait for wgpu::Surface {
    #[inline]
    fn configure(&self, device: &crate::Device, config: &crate::SurfaceConfiguration) {
        let device = unsafe { &*(device.0.as_ref() as *const _ as *const wgpu::Device) };

        self.configure(device, config);
    }

    fn get_current_frame(&self) -> Result<SurfaceFrame, SurfaceError> {
        let frame = self.get_current_frame();

        match frame {
            Ok(frame) => Ok(SurfaceFrame {
                output: SurfaceTexture(Box::new(frame.output)),
                suboptimal: frame.suboptimal,
            }),
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
    pub fn get_current_frame(&self) -> Result<SurfaceFrame, SurfaceError> {
        self.0.get_current_frame()
    }
}
