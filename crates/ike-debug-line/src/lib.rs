mod debug_line;
mod node;

pub use debug_line::*;
pub use node::*;

use ike_core::*;
use ike_render::*;

pub mod render_graph {
    pub const DEBUG_LINE_NODE: &str = "debug_line_node";
}

pub struct DebugLinePlugin;

impl Plugin for DebugLinePlugin {
    #[inline]
    fn build(self, app: &mut AppBuilder) {
        let mut render_graph = app.world().write_resource::<RenderGraph>().unwrap();

        render_graph.insert_node(DebugLineNode::new(), render_graph::DEBUG_LINE_NODE);
        render_graph
            .insert_slot_edge(
                ike_render::render_graph::INPUT,
                ViewInputNode::TARGET,
                render_graph::DEBUG_LINE_NODE,
                DebugLineNode::TARGET,
            )
            .unwrap();
        render_graph
            .insert_slot_edge(
                ike_render::render_graph::INPUT,
                ViewInputNode::CAMERA,
                render_graph::DEBUG_LINE_NODE,
                DebugLineNode::CAMERA,
            )
            .unwrap();
    }
}
