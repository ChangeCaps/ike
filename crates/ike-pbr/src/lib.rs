mod material;
mod pbr_node;

pub use material::*;
pub use pbr_node::*;

use ike_app::{App, Plugin};
use ike_assets::AddAsset;
use ike_render::{input, RenderGraph, RenderStage, TextureNode};

pub mod node {
    pub const PBR_NODE: &str = "pbr_node";
}

pub struct PbrPlugin;

impl Plugin for PbrPlugin {
    fn build(self, app: &mut App) {
        app.world.init_resource::<PbrResources>();
        app.add_asset::<PbrMaterial>();
        app.add_asset::<MaterialBinding>();

        app.add_system_to_stage(MaterialBinding::system, RenderStage::PreRender);

        let mut render_graph = app.world.resource_mut::<RenderGraph>();

        let input_node = render_graph.input_node().expect("input node should be set");
        render_graph.add_node(node::PBR_NODE, PbrNode::default());

        render_graph
            .add_node_edge(ike_render::node::DEPENDENCIES, node::PBR_NODE)
            .unwrap();

        render_graph
            .add_node_edge(node::PBR_NODE, ike_post_processing::node::DEPENDENCIES)
            .unwrap();

        render_graph
            .add_node_edge(node::PBR_NODE, ike_debug::node::DEBUG_LINE)
            .unwrap();

        render_graph
            .add_slot_edge(
                ike_render::node::HDR,
                TextureNode::TEXTURE,
                node::PBR_NODE,
                PbrNode::RENDER_TARGET,
            )
            .unwrap();

        render_graph
            .add_slot_edge(
                ike_render::node::HDR_MSAA,
                TextureNode::TEXTURE,
                node::PBR_NODE,
                PbrNode::MSAA_TEXTURE,
            )
            .unwrap();

        render_graph
            .add_slot_edge(
                ike_render::node::MSAA_DEPTH,
                TextureNode::TEXTURE,
                node::PBR_NODE,
                PbrNode::DEPTH,
            )
            .unwrap();

        render_graph
            .add_slot_edge(input_node, input::CAMERA, node::PBR_NODE, PbrNode::CAMERA)
            .unwrap();
    }
}
