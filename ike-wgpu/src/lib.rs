mod bind_group;
mod buffer;
mod command_encoder;
mod compute;
mod device;
mod pipeline;
mod queue;
mod render_pass;
mod sampler;
mod shader;
mod surface;
mod texture;

pub use wgpu_types::*;

pub use bind_group::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindingResource, BufferBinding,
};
pub use buffer::{Buffer, BufferInitDescriptor, BufferSlice, BufferView, MapMode};
pub use command_encoder::{CommandBuffer, CommandEncoder};
pub use compute::{ComputePass, ComputePassDescriptor, ComputePipeline, ComputePipelineDescriptor};
pub use device::{Device, Maintain};
pub use pipeline::{
    FragmentState, PipelineLayout, PipelineLayoutDescriptor, RenderPipeline,
    RenderPipelineDescriptor, VertexBufferLayout, VertexState,
};
pub use queue::Queue;
pub use render_pass::{
    LoadOp, Operations, RenderPass, RenderPassColorAttachment, RenderPassDepthStencilAttachment,
    RenderPassDescriptor,
};
pub use sampler::{Sampler, SamplerDescriptor};
pub use shader::{ShaderModule, ShaderModuleDescriptor, ShaderSource};
pub use surface::{Surface, SurfaceError, SurfaceFrame, SurfaceTexture};
pub use texture::{Texture, TextureView, TextureViewDescriptor};

#[macro_export]
macro_rules! include_wgsl {
    ($($token:tt)*) => {
        {
            $crate::ShaderModuleDescriptor {
                label: Some($($token)*),
                source: $crate::ShaderSource::Wgsl(include_str!($($token)*).into()),
            }
        }
    };
}
