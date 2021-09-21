use crate::{BindGroupLayout, ShaderModule};

pub struct PipelineLayoutDescriptor<'a> {
    pub label: Option<&'a str>,
    pub bind_group_layouts: &'a [&'a BindGroupLayout],
    pub push_constant_ranges: &'a [crate::PushConstantRange],
}

pub(crate) unsafe trait PipelineLayoutTrait: std::fmt::Debug {}

#[cfg(feature = "wgpu")]
unsafe impl PipelineLayoutTrait for wgpu::PipelineLayout {}

#[derive(Debug)]
pub struct PipelineLayout(pub(crate) Box<dyn PipelineLayoutTrait>);

pub struct VertexBufferLayout<'a> {
    pub array_stride: u64,
    pub step_mode: crate::VertexStepMode,
    pub attributes: &'a [crate::VertexAttribute],
}

pub struct VertexState<'a> {
    pub module: &'a ShaderModule,
    pub entry_point: &'a str,
    pub buffers: &'a [VertexBufferLayout<'a>],
}

pub struct FragmentState<'a> {
    pub module: &'a ShaderModule,
    pub entry_point: &'a str,
    pub targets: &'a [crate::ColorTargetState],
}

pub struct RenderPipelineDescriptor<'a> {
    pub label: Option<&'a str>,
    pub layout: Option<&'a PipelineLayout>,
    pub vertex: VertexState<'a>,
    pub fragment: Option<FragmentState<'a>>,
    pub primitive: crate::PrimitiveState,
    pub multisample: crate::MultisampleState,
    pub depth_stencil: Option<crate::DepthStencilState>,
}

pub(crate) unsafe trait RenderPipelineTrait {}

#[cfg(feature = "wgpu")]
unsafe impl RenderPipelineTrait for wgpu::RenderPipeline {}

pub struct RenderPipeline(pub(crate) Box<dyn RenderPipelineTrait>);
