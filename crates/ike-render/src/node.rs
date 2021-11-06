use glam::UVec2;
use ike_core::WorldRef;

use crate::{
    render_device, wgpu, Camera, EdgeSlotInfo, GraphError, NodeEdge, NodeInput, RenderNode,
    RenderTarget, RenderTexture,
};

#[derive(Default)]
pub struct ViewInputNode;

impl ViewInputNode {
    pub const TARGET: &'static str = "target";
    pub const CAMERA: &'static str = "camera";
}

impl RenderNode for ViewInputNode {
    fn output(&self) -> Vec<EdgeSlotInfo> {
        vec![
            EdgeSlotInfo::new::<RenderTexture>(Self::TARGET),
            EdgeSlotInfo::new::<Camera>(Self::CAMERA),
        ]
    }

    fn run(
        &mut self,
        _encoder: &mut ike_wgpu::CommandEncoder,
        _world: &WorldRef,
        _input: &NodeInput,
        _output: &mut NodeEdge,
    ) -> Result<(), GraphError> {
        Ok(())
    }
}

#[derive(Default)]
pub struct DepthTextureNode {
    pub target: Option<RenderTarget>,
    pub size: Option<UVec2>,
}

impl DepthTextureNode {
    pub const TARGET: &'static str = "target";
    pub const DEPTH: &'static str = "depth";
}

impl RenderNode for DepthTextureNode {
    fn input(&self) -> Vec<EdgeSlotInfo> {
        vec![EdgeSlotInfo::new::<RenderTexture>(Self::TARGET)]
    }

    fn output(&self) -> Vec<EdgeSlotInfo> {
        vec![EdgeSlotInfo::new::<wgpu::TextureView>(Self::DEPTH)]
    }

    fn run(
        &mut self,
        _encoder: &mut crate::wgpu::CommandEncoder,
        _world: &WorldRef,
        input: &NodeInput,
        output: &mut NodeEdge,
    ) -> Result<(), GraphError> {
        let target = input.get::<RenderTexture>(Self::TARGET)?;

        if self.target != Some(target.target()) || self.size != Some(target.size) {
            self.target = Some(target.target());
            self.size = Some(target.size);

            let texture = render_device().create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: target.size.x,
                    height: target.size.y,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: target.samples,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            });

            let view = texture.create_view(&Default::default());

            output.set(Self::DEPTH, view)?;
        }

        Ok(())
    }
}
