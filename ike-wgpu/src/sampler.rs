use std::num::NonZeroU8;

#[derive(Default)]
pub struct SamplerDescriptor<'a> {
    pub label: Option<&'a str>,
    pub address_mode_u: crate::AddressMode,
    pub address_mode_v: crate::AddressMode,
    pub address_mode_w: crate::AddressMode,
    pub mag_filter: crate::FilterMode,
    pub min_filter: crate::FilterMode,
    pub mipmap_filter: crate::FilterMode,
    pub lod_min_clamp: f32,
    pub lod_max_clamp: f32,
    pub compare: Option<crate::CompareFunction>,
    pub anisotropy_clamp: Option<NonZeroU8>,
    pub border_color: Option<crate::SamplerBorderColor>,
}

pub(crate) unsafe trait SamplerTrait {}

#[cfg(feature = "wgpu")]
unsafe impl SamplerTrait for wgpu::Sampler {}

pub struct Sampler(pub(crate) Box<dyn SamplerTrait>);
