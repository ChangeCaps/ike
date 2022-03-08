mod material;
mod pbr_node;

pub use material::*;
pub use pbr_node::*;

use ike_app::{App, Plugin};
use ike_assets::AddAsset;
use ike_render::{input, RenderGraph, TextureNode};

pub mod node {
    pub const PBR_NODE: &str = "pbr_node";
}

pub struct PbrPlugin;

impl Plugin for PbrPlugin {
    fn build(self, app: &mut App) {
        app.world.init_resource::<PbrResources>();
        app.add_asset::<PbrMaterial>();

        let mut render_graph = app.world.resource_mut::<RenderGraph>();

        let input_node = render_graph.input_node().expect("input node should be set");
        render_graph.add_node(node::PBR_NODE, PbrNode::default());

        render_graph
            .add_node_edge(ike_render::node::DEPENDENCIES, node::PBR_NODE)
            .unwrap();

        render_graph
            .add_slot_edge(
                input_node,
                input::SURFACE_TEXTURE,
                node::PBR_NODE,
                PbrNode::RENDER_TARGET,
            )
            .unwrap();

        render_graph
            .add_slot_edge(
                ike_render::node::MSAA,
                TextureNode::TEXTURE,
                node::PBR_NODE,
                PbrNode::MSAA_TEXTURE,
            )
            .unwrap();

        render_graph
            .add_slot_edge(
                ike_render::node::DEPTH,
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
