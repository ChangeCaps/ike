#[cfg(feature = "render")]
mod debug_line;

#[cfg(feature = "render")]
pub use debug_line::*;

use ike_app::{App, Plugin};
#[cfg(feature = "render")]
use ike_render::{input, RenderGraph, TextureNode};

#[cfg(feature = "render")]
pub mod node {
    pub const DEBUG_LINE: &str = "debug_line";
}

#[derive(Default)]
pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    #[allow(unused)]
    fn build(self, app: &mut App) {
        #[cfg(feature = "render")]
        app.init_resource::<DebugLinePipeline>();

        #[cfg(feature = "render")]
        let mut render_graph = app.world.resource_mut::<RenderGraph>();

        #[cfg(feature = "render")]
        let input_node = render_graph.input_node().unwrap();

        #[cfg(feature = "render")]
        render_graph.add_node(node::DEBUG_LINE, DebugLineNode::default());

        #[cfg(feature = "render")]
        render_graph
            .add_slot_edge(
                input_node,
                input::CAMERA,
                node::DEBUG_LINE,
                DebugLineNode::CAMERA,
            )
            .unwrap();

        #[cfg(feature = "render")]
        render_graph
            .add_slot_edge(
                input_node,
                input::SURFACE_TEXTURE,
                node::DEBUG_LINE,
                DebugLineNode::TARGET,
            )
            .unwrap();

        #[cfg(feature = "render")]
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
