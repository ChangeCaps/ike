use std::num::NonZeroU64;

pub struct BindGroupLayoutDescriptor<'a> {
    pub label: Option<&'a str>,
    pub entries: &'a [crate::BindGroupLayoutEntry],
}

pub(crate) unsafe trait BindGroupLayoutTrait {}

#[cfg(feature = "wgpu")]
unsafe impl BindGroupLayoutTrait for wgpu::BindGroupLayout {}

pub struct BindGroupLayout(pub(crate) Box<dyn BindGroupLayoutTrait>);

pub struct BufferBinding<'a> {
    pub buffer: &'a crate::Buffer,
    pub offset: u64,
    pub size: Option<NonZeroU64>,
}

pub enum BindingResource<'a> {
    Buffer(BufferBinding<'a>),
    BufferArray(&'a [BufferBinding<'a>]),
    Sampler(&'a crate::Sampler),
    TextureView(&'a crate::TextureView),
    TextureViewArray(&'a [&'a crate::TextureView]),
}

pub struct BindGroupEntry<'a> {
    pub binding: u32,
    pub resource: crate::BindingResource<'a>,
}

pub struct BindGroupDescriptor<'a> {
    pub label: Option<&'a str>,
    pub layout: &'a crate::BindGroupLayout,
    pub entries: &'a [BindGroupEntry<'a>],
}

pub(crate) unsafe trait BindGroupTrait {}

#[cfg(feature = "wgpu")]
unsafe impl BindGroupTrait for wgpu::BindGroup {}

pub struct BindGroup(pub(crate) Box<dyn BindGroupTrait>);
