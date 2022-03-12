mod bloom;
mod tone_mapping;

pub use bloom::*;
pub use tone_mapping::*;

use ike_app::{App, Plugin};
use ike_render::{input, EmptyNode, RenderGraph, TextureNode};

pub mod node {
    pub const DEPENDENCIES: &str = "post_processing_dependencies";
    pub const BLOOM: &str = "bloom";
    pub const TONE_MAPPING: &str = "tone_mapping";
}

#[derive(Default)]
pub struct PostProcessingPlugin;

impl Plugin for PostProcessingPlugin {
    fn build(self, app: &mut App) {
        app.init_resource::<BloomPipeline>();
        app.init_resource::<ToneMappingPipeline>();

        let mut render_graph = app.world.resource_mut::<RenderGraph>();

        let input_node = render_graph.input_node().expect("input node should be set");

        render_graph.add_node(node::DEPENDENCIES, EmptyNode);
        render_graph.add_node(node::TONE_MAPPING, ToneMappingNode::default());
        render_graph.add_node(node::BLOOM, BloomNode::default());

        render_graph
            .add_node_edge(ike_render::node::DEPENDENCIES, node::DEPENDENCIES)
            .unwrap();

        render_graph
            .add_node_edge(node::DEPENDENCIES, node::BLOOM)
            .unwrap();

        render_graph
            .add_node_edge(node::DEPENDENCIES, node::TONE_MAPPING)
            .unwrap();

        render_graph
            .add_slot_edge(
                ike_render::node::HDR,
                TextureNode::TEXTURE,
                node::BLOOM,
                BloomNode::TARGET,
            )
            .unwrap();

        render_graph
            .add_slot_edge(
                node::BLOOM,
                BloomNode::OUTPUT,
                node::TONE_MAPPING,
                ToneMappingNode::HDR_TARGET,
            )
            .unwrap();

        render_graph
            .add_slot_edge(
                input_node,
                input::SURFACE_TEXTURE,
                node::TONE_MAPPING,
                ToneMappingNode::TARGET,
            )
            .unwrap();
    }
}
