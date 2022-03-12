use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use ike_util::Id;
use wgpu::util::DeviceExt;

pub use wgpu::include_wgsl;

pub type BufferUsages = wgpu::BufferUsages;
pub type BufferDescriptor<'a> = wgpu::BufferDescriptor<'a>;
pub type BufferInitDescriptor<'a> = wgpu::util::BufferInitDescriptor<'a>;
pub type TextureViewDescriptor<'a> = wgpu::TextureViewDescriptor<'a>;
pub type SurfaceConfiguration = wgpu::SurfaceConfiguration;
pub type PresentMode = wgpu::PresentMode;
pub type SurfaceError = wgpu::SurfaceError;
pub type TextureDescriptor<'a> = wgpu::TextureDescriptor<'a>;
pub type TextureUsages = wgpu::TextureUsages;
pub type TextureFormat = wgpu::TextureFormat;
pub type TextureDimension = wgpu::TextureDimension;
pub type Extent3d = wgpu::Extent3d;
pub type Sampler = wgpu::Sampler;
pub type SamplerDescriptor<'a> = wgpu::SamplerDescriptor<'a>;
pub type FilterMode = wgpu::FilterMode;
pub type CompareFunction = wgpu::CompareFunction;
pub type CommandEncoder = wgpu::CommandEncoder;
pub type CommandBuffer = wgpu::CommandBuffer;
pub type CommandEncoderDescriptor<'a> = wgpu::CommandEncoderDescriptor<'a>;
pub type ImageCopyTexture<'a> = wgpu::ImageCopyTexture<'a>;
pub type ImageCopyBuffer<'a> = wgpu::ImageCopyBuffer<'a>;
pub type Origin3d = wgpu::Origin3d;
pub type RenderPass<'a> = wgpu::RenderPass<'a>;
pub type RenderPassDescriptor<'a, 'b> = wgpu::RenderPassDescriptor<'a, 'b>;
pub type RenderPassColorAttachment<'a> = wgpu::RenderPassColorAttachment<'a>;
pub type RenderPassDepthStencilAttachment<'a> = wgpu::RenderPassDepthStencilAttachment<'a>;
pub type IndexFormat = wgpu::IndexFormat;
pub type Operations<T> = wgpu::Operations<T>;
pub type LoadOp<T> = wgpu::LoadOp<T>;
pub type RawColor = wgpu::Color;
pub type BindGroup = wgpu::BindGroup;
pub type BindGroupEntry<'a> = wgpu::BindGroupEntry<'a>;
pub type BindGroupLayout = wgpu::BindGroupLayout;
pub type BindGroupLayoutEntry = wgpu::BindGroupLayoutEntry;
pub type BindGroupDescriptor<'a> = wgpu::BindGroupDescriptor<'a>;
pub type BindGroupLayoutDescriptor<'a> = wgpu::BindGroupLayoutDescriptor<'a>;
pub type BindingType = wgpu::BindingType;
pub type BufferBindingType = wgpu::BufferBindingType;
pub type BindingResource<'a> = wgpu::BindingResource<'a>;
pub type SamplerBindingType = wgpu::SamplerBindingType;
pub type ShaderStages = wgpu::ShaderStages;
pub type StorageTextureAccess = wgpu::StorageTextureAccess;
pub type TextureSampleType = wgpu::TextureSampleType;
pub type TextureViewDimension = wgpu::TextureViewDimension;
pub type PipelineLayout = wgpu::PipelineLayout;
pub type PipelineLayoutDescriptor<'a> = wgpu::PipelineLayoutDescriptor<'a>;
pub type RenderPipeline = wgpu::RenderPipeline;
pub type RenderPipelineDescriptor<'a> = wgpu::RenderPipelineDescriptor<'a>;
pub type BlendComponent = wgpu::BlendComponent;
pub type BlendFactor = wgpu::BlendFactor;
pub type BlendOperation = wgpu::BlendOperation;
pub type DepthStencilState = wgpu::DepthStencilState;
pub type StencilState = wgpu::StencilState;
pub type DepthBiasState = wgpu::DepthBiasState;
pub type VertexState<'a> = wgpu::VertexState<'a>;
pub type VertexBufferLayout<'a> = wgpu::VertexBufferLayout<'a>;
pub type VertexStepMode = wgpu::VertexStepMode;
pub type VertexAttribute = wgpu::VertexAttribute;
pub type VertexFormat = wgpu::VertexFormat;
pub type FragmentState<'a> = wgpu::FragmentState<'a>;
pub type ColorTargetState = wgpu::ColorTargetState;
pub type BlendState = wgpu::BlendState;
pub type ColorWrites = wgpu::ColorWrites;
pub type MultisampleState = wgpu::MultisampleState;
pub type PrimitiveState = wgpu::PrimitiveState;
pub type PrimitiveTopology = wgpu::PrimitiveTopology;
pub type Face = wgpu::Face;
pub type PolygonMode = wgpu::PolygonMode;
pub type FrontFace = wgpu::FrontFace;
pub type ShaderModule = wgpu::ShaderModule;
pub type ShaderModuleDescriptor<'a> = wgpu::ShaderModuleDescriptor<'a>;
pub type ComputePipeline = wgpu::ComputePipeline;
pub type ComputePipelineDescriptor<'a> = wgpu::ComputePipelineDescriptor<'a>;
pub type ComputePassDescriptor<'a> = wgpu::ComputePassDescriptor<'a>;
pub type ComputePass<'a> = wgpu::ComputePass<'a>;

pub type RawTexture = wgpu::Texture;
pub type RawTextureView = wgpu::TextureView;
pub type RawSurface = wgpu::Surface;
pub type RawSurfaceTexture = wgpu::SurfaceTexture;
pub type RawBuffer = wgpu::Buffer;
pub type RawRenderDevice = wgpu::Device;
pub type RawRenderQueue = wgpu::Queue;

pub type TextureId = Id<RawTexture>;
pub type TextureViewId = Id<RawTextureView>;
pub type SurfaceTextureId = Id<RawSurfaceTexture>;
pub type BufferId = Id<RawBuffer>;
pub type RenderDeviceId = Id<RawRenderDevice>;
pub type RenderQueueId = Id<RawRenderQueue>;

#[derive(Clone, Debug)]
pub struct RenderDevice {
    raw: Arc<RawRenderDevice>,
}

impl RenderDevice {
    pub fn from_raw(raw: RawRenderDevice) -> Self {
        Self { raw: Arc::new(raw) }
    }

    pub fn id(&self) -> RenderDeviceId {
        Id::from_arc(&self.raw)
    }

    pub fn raw(&self) -> &RawRenderDevice {
        &self.raw
    }

    pub fn create_buffer(&self, desc: &BufferDescriptor<'_>) -> Buffer {
        let raw = self.raw.create_buffer(desc);
        Buffer::from_raw(raw)
    }

    pub fn create_buffer_init(&self, desc: &BufferInitDescriptor<'_>) -> Buffer {
        let raw = self.raw.create_buffer_init(desc);
        Buffer::from_raw(raw)
    }

    pub fn create_texture(&self, desc: &TextureDescriptor<'_>) -> Texture {
        let raw = self.raw.create_texture(desc);
        Texture::from_raw(raw, desc.format, desc.size.width, desc.size.height)
    }

    pub fn create_texture_with_data(
        &self,
        queue: &RenderQueue,
        desc: &TextureDescriptor<'_>,
        data: &[u8],
    ) -> Texture {
        let raw = self.raw.create_texture_with_data(queue.raw(), desc, data);
        Texture::from_raw(raw, desc.format, desc.size.width, desc.size.height)
    }

    pub fn create_sampler(&self, desc: &SamplerDescriptor<'_>) -> Sampler {
        self.raw.create_sampler(desc)
    }

    pub fn create_command_encoder(&self, desc: &CommandEncoderDescriptor<'_>) -> CommandEncoder {
        self.raw.create_command_encoder(desc)
    }

    pub fn create_bind_group_layout(
        &self,
        desc: &BindGroupLayoutDescriptor<'_>,
    ) -> BindGroupLayout {
        self.raw.create_bind_group_layout(desc)
    }

    pub fn create_bind_group(&self, desc: &BindGroupDescriptor<'_>) -> BindGroup {
        self.raw.create_bind_group(desc)
    }

    pub fn create_shader_module(&self, desc: &ShaderModuleDescriptor<'_>) -> ShaderModule {
        self.raw.create_shader_module(desc)
    }

    pub fn create_pipeline_layout(&self, desc: &PipelineLayoutDescriptor<'_>) -> PipelineLayout {
        self.raw.create_pipeline_layout(desc)
    }

    pub fn create_render_pipeline(&self, desc: &RenderPipelineDescriptor<'_>) -> RenderPipeline {
        self.raw.create_render_pipeline(desc)
    }

    pub fn create_compute_pipeline(&self, desc: &ComputePipelineDescriptor<'_>) -> ComputePipeline {
        self.raw.create_compute_pipeline(desc)
    }
}

#[derive(Clone, Debug)]
pub struct RenderQueue {
    raw: Arc<RawRenderQueue>,
}

impl RenderQueue {
    pub fn from_raw(raw: RawRenderQueue) -> Self {
        Self { raw: Arc::new(raw) }
    }

    pub fn id(&self) -> RenderQueueId {
        Id::from_arc(&self.raw)
    }

    pub fn raw(&self) -> &RawRenderQueue {
        &self.raw
    }

    pub fn write_buffer(&self, buffer: &Buffer, offset: u64, data: &[u8]) {
        self.raw.write_buffer(buffer.raw(), offset, data);
    }

    pub fn submit(&self, buffer: Vec<CommandBuffer>) {
        self.raw.submit(buffer.into_iter());
    }

    pub fn submit_one(&self, buffer: CommandBuffer) {
        self.raw.submit(std::iter::once(buffer));
    }
}

#[derive(Debug)]
pub struct Surface {
    raw: Arc<RawSurface>,
    device: RenderDevice,
    width: u32,
    height: u32,
    usage: TextureUsages,
    format: TextureFormat,
    present_mode: PresentMode,
    config_surface: AtomicBool,
}

impl Surface {
    pub fn from_raw(raw: RawSurface, device: &RenderDevice) -> Self {
        Self {
            raw: Arc::new(raw),
            device: device.clone(),
            width: 1,
            height: 1,
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: TextureFormat::Bgra8UnormSrgb,
            present_mode: PresentMode::Fifo,
            config_surface: AtomicBool::new(true),
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width.max(1);
        self.height = height.max(1);
        *self.config_surface.get_mut() = true;
    }

    pub fn format(&self) -> TextureFormat {
        self.format
    }

    pub fn config_surface(&self) {
        self.raw.configure(
            &self.device.raw(),
            &SurfaceConfiguration {
                usage: self.usage,
                format: self.format,
                width: self.width,
                height: self.height,
                present_mode: self.present_mode,
            },
        );
    }

    pub fn get_current_texture(&self) -> Result<SurfaceTexture, SurfaceError> {
        if self.config_surface.swap(false, Ordering::AcqRel) {
            self.config_surface();
        }

        Ok(SurfaceTexture::from_raw(
            self.raw.get_current_texture()?,
            self.format,
            self.width,
            self.height,
        ))
    }
}

#[derive(Debug)]
pub struct SurfaceTexture {
    raw: RawSurfaceTexture,
    format: TextureFormat,
    width: u32,
    height: u32,
}

impl SurfaceTexture {
    pub fn from_raw(
        raw: RawSurfaceTexture,
        format: TextureFormat,
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            raw,
            format,
            width,
            height,
        }
    }

    pub fn raw(&self) -> &RawSurfaceTexture {
        &self.raw
    }

    pub fn format(&self) -> TextureFormat {
        self.format
    }

    pub fn create_view(&self, desc: &TextureViewDescriptor<'_>) -> TextureView {
        TextureView::from_raw(
            self.raw.texture.create_view(desc),
            desc.format.unwrap_or(self.format),
            self.width,
            self.height,
        )
    }

    pub fn present(self) {
        self.raw.present();
    }
}
#[derive(Clone, Debug)]
pub struct Texture {
    raw: Arc<RawTexture>,
    format: TextureFormat,
    width: u32,
    height: u32,
}

impl Texture {
    pub fn from_raw(raw: RawTexture, format: TextureFormat, width: u32, height: u32) -> Self {
        Self {
            raw: Arc::new(raw),
            format,
            width,
            height,
        }
    }

    pub fn id(&self) -> TextureId {
        Id::from_arc(&self.raw)
    }

    pub fn format(&self) -> TextureFormat {
        self.format
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn create_view(&self, desc: &TextureViewDescriptor<'_>) -> TextureView {
        TextureView::from_raw(
            self.raw.create_view(desc),
            desc.format.unwrap_or(self.format),
            self.width,
            self.height,
        )
    }

    pub fn raw(&self) -> &RawTexture {
        &self.raw
    }
}

#[derive(Clone, Debug)]
pub struct TextureView {
    raw: Arc<RawTextureView>,
    format: TextureFormat,
    width: u32,
    height: u32,
}

impl TextureView {
    pub fn from_raw(raw: RawTextureView, format: TextureFormat, width: u32, height: u32) -> Self {
        Self {
            raw: Arc::new(raw),
            format,
            width,
            height,
        }
    }

    pub fn id(&self) -> TextureViewId {
        Id::from_arc(&self.raw)
    }

    pub fn format(&self) -> TextureFormat {
        self.format
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn raw(&self) -> &RawTextureView {
        &self.raw
    }
}

#[derive(Clone, Debug)]
pub struct Buffer {
    raw: Arc<RawBuffer>,
}

impl Buffer {
    pub fn from_raw(raw: RawBuffer) -> Self {
        Self { raw: Arc::new(raw) }
    }

    pub fn id(&self) -> BufferId {
        Id::from_arc(&self.raw)
    }

    pub fn raw(&self) -> &RawBuffer {
        &self.raw
    }
}
