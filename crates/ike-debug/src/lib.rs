mod debug_line;

pub use debug_line::*;

use ike_app::{App, Plugin};
use ike_render::{input, RenderGraph, TextureNode};

pub mod node {
    pub const DEBUG_LINE: &str = "debug_line";
}

#[derive(Default)]
pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(self, app: &mut App) {
        app.init_resource::<DebugLinePipeline>();

        let mut render_graph = app.world.resource_mut::<RenderGraph>();

        let input_node = render_graph.input_node().unwrap();

        render_graph.add_node(node::DEBUG_LINE, DebugLineNode::default());

        render_graph
            .add_slot_edge(
                input_node,
                input::CAMERA,
                node::DEBUG_LINE,
                DebugLineNode::CAMERA,
            )
            .unwrap();

        render_graph
            .add_slot_edge(
                input_node,
                input::SURFACE_TEXTURE,
                node::DEBUG_LINE,
                DebugLineNode::TARGET,
            )
            .unwrap();

        render_graph
            .add_slot_edge(
                ike_render::node::DEPTH,
                TextureNode::TEXTURE,
                node::DEBUG_LINE,
                DebugLineNode::DEPTH,
            )
            .unwrap();
    }
}
