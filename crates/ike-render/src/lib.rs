mod camera;
mod color;
mod edge;
mod image;
mod image_loader;
pub mod mesh;
mod mesh_binding;
mod mesh_tool;
mod render_graph;
mod render_node;
mod render_system;
mod resources;
mod run_render_graph;
mod settings;
mod shapes;
mod slot;
mod texture_node;

pub use self::image::*;
pub use camera::*;
pub use color::*;
pub use edge::*;
pub use image_loader::*;
pub use mesh::{Mesh, MeshBuffers};
pub use mesh_binding::*;
pub use mesh_tool::*;
pub use render_graph::*;
pub use render_node::*;
pub use render_system::*;
pub use resources::*;
pub use settings::*;
pub use slot::*;
pub use texture_node::*;

use ike_app::{App, CoreStage, Plugin};
use ike_assets::AddAsset;
use ike_ecs::{IntoExclusiveSystem, StageLabel};

#[derive(StageLabel, Clone, Copy, Debug, Hash)]
pub enum RenderStage {
    PreRender,
    Render,
    PostRender,
}

pub mod input {
    pub const SURFACE_TEXTURE: &str = "surface_texture";
    pub const CAMERA: &str = "camera";
}

pub mod node {
    pub const DEPENDENCIES: &str = "dependencies";
    pub const DEPTH: &str = "depth";
    pub const MSAA_DEPTH: &str = "msaa_depth";
    pub const HDR_MSAA: &str = "hdr_msaa";
    pub const MSAA: &str = "msaa";
    pub const HDR: &str = "hdr";
}

#[derive(Default)]
pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(self, app: &mut App) {
        app.register::<Camera>();

        app.init_resource::<RenderSettings>();

        app.add_asset::<Mesh>();
        app.add_asset::<MeshBuffers>();
        app.add_asset::<Image>();
        app.add_asset::<ImageTexture>();

        app.add_asset_loader(ImageLoader);

        app.add_stage_after(RenderStage::PreRender, CoreStage::PostUpdate);
        app.add_stage_after(RenderStage::Render, RenderStage::PreRender);
        app.add_stage_after(RenderStage::PostRender, RenderStage::Render);

        app.add_system_to_stage(image_texture_system, RenderStage::PreRender);
        app.add_system_to_stage(render_system.exclusive_system(), RenderStage::Render);

        let mut render_graph = RenderGraph::default();

        let input_node = render_graph.set_input(vec![
            SlotInfo::new::<TextureView>(input::SURFACE_TEXTURE),
            SlotInfo::new::<RawCamera>(input::CAMERA),
        ]);

        render_graph.add_node(node::DEPENDENCIES, EmptyNode);
        render_graph
            .add_node_edge(input_node, node::DEPENDENCIES)
            .unwrap();

        render_graph.add_node(
            node::DEPTH,
            TextureNode::new(TextureDescriptor {
                label: Some("ike_depth_texture"),
                size: Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                dimension: TextureDimension::D2,
                mip_level_count: 1,
                sample_count: 1,
                format: TextureFormat::Depth32Float,
                usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            }),
        );

        render_graph.add_node(
            node::MSAA_DEPTH,
            TextureNode::new(TextureDescriptor {
                label: Some("ike_depth_texture"),
                size: Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                dimension: TextureDimension::D2,
                mip_level_count: 1,
                sample_count: 4,
                format: TextureFormat::Depth32Float,
                usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            }),
        );

        render_graph.add_node(
            node::MSAA,
            TextureNode::new(TextureDescriptor {
                label: Some("ike_msaa_texture"),
                size: Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                dimension: TextureDimension::D2,
                mip_level_count: 1,
                sample_count: 4,
                format: TextureFormat::Bgra8UnormSrgb,
                usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            }),
        );

        render_graph.add_node(
            node::HDR,
            TextureNode::new(TextureDescriptor {
                label: Some("ike_hdr_texture"),
                size: Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                dimension: TextureDimension::D2,
                mip_level_count: 1,
                sample_count: 1,
                format: TextureFormat::Rgba32Float,
                usage: TextureUsages::RENDER_ATTACHMENT
                    | TextureUsages::TEXTURE_BINDING
                    | TextureUsages::STORAGE_BINDING,
            }),
        );

        render_graph.add_node(
            node::HDR_MSAA,
            TextureNode::new(TextureDescriptor {
                label: Some("ike_msaa_texture"),
                size: Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                dimension: TextureDimension::D2,
                mip_level_count: 1,
                sample_count: 4,
                format: TextureFormat::Rgba32Float,
                usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            }),
        );

        render_graph
            .add_slot_edge(
                input_node,
                input::SURFACE_TEXTURE,
                node::DEPTH,
                TextureNode::TEXTURE,
            )
            .unwrap();

        render_graph
            .add_slot_edge(
                input_node,
                input::SURFACE_TEXTURE,
                node::MSAA_DEPTH,
                TextureNode::TEXTURE,
            )
            .unwrap();

        render_graph
            .add_slot_edge(
                input_node,
                input::SURFACE_TEXTURE,
                node::MSAA,
                TextureNode::TEXTURE,
            )
            .unwrap();

        render_graph
            .add_slot_edge(
                input_node,
                input::SURFACE_TEXTURE,
                node::HDR_MSAA,
                TextureNode::TEXTURE,
            )
            .unwrap();

        render_graph
            .add_slot_edge(
                input_node,
                input::SURFACE_TEXTURE,
                node::HDR,
                TextureNode::TEXTURE,
            )
            .unwrap();

        app.insert_resource(render_graph);
    }
}
