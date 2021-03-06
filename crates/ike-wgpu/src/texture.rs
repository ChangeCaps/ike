use std::num::NonZeroU32;

pub(crate) unsafe trait TextureTrait: Send + Sync + std::fmt::Debug {
    fn create_view(&self, desc: &TextureViewDescriptor) -> TextureView;
}

#[cfg(feature = "wgpu")]
unsafe impl TextureTrait for wgpu::Texture {
    #[inline]
    fn create_view(&self, desc: &TextureViewDescriptor) -> TextureView {
        let view = self.create_view(&wgpu::TextureViewDescriptor {
            label: desc.label,
            format: desc.format,
            dimension: desc.dimension,
            aspect: desc.aspect,
            base_mip_level: desc.base_mip_level,
            mip_level_count: desc.mip_level_count,
            base_array_layer: desc.base_array_layer,
            array_layer_count: desc.array_layer_count,
        });

        TextureView(Box::new(view))
    }
}

#[repr(transparent)]
#[derive(Debug)]
pub struct Texture(pub(crate) *const dyn TextureTrait);

impl Texture {
    #[inline]
    pub fn create_view(&self, desc: &TextureViewDescriptor) -> TextureView {
        unsafe { &*self.0 }.create_view(desc)
    }
}

unsafe impl Sync for Texture {}
unsafe impl Send for Texture {}

impl Drop for Texture {
    #[inline]
    fn drop(&mut self) {
        unsafe { Box::from_raw(self.0 as *mut dyn TextureTrait) };
    }
}

#[derive(Default)]
pub struct TextureViewDescriptor {
    pub label: Option<&'static str>,
    pub format: Option<crate::TextureFormat>,
    pub dimension: Option<crate::TextureViewDimension>,
    pub aspect: crate::TextureAspect,
    pub base_mip_level: u32,
    pub mip_level_count: Option<NonZeroU32>,
    pub base_array_layer: u32,
    pub array_layer_count: Option<NonZeroU32>,
}

pub(crate) unsafe trait TextureViewTrait: std::fmt::Debug + Send + Sync {}

#[cfg(feature = "wgpu")]
unsafe impl TextureViewTrait for wgpu::TextureView {}

#[derive(Debug)]
pub struct TextureView(pub(crate) Box<dyn TextureViewTrait>);
