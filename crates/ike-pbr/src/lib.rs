mod light;
mod material;
mod node;
mod sky;

pub use light::*;
pub use material::*;
pub use node::*;
pub use sky::*;

use ike_assets::AssetAppBuilderExt;
use ike_core::*;
use ike_render::*;

pub mod render_graph {
    pub const SKY_NODE: &str = "sky_node";
    pub const PBR_NODE: &str = "pbr_node";
}

pub struct PbrPlugin;

impl Plugin for PbrPlugin {
    #[inline]
    fn build(self, app: &mut AppBuilder) {
        app.add_asset::<PbrMaterial>();

        let mut render_graph = app.world().write_resource::<RenderGraph>().unwrap();

        render_graph.insert_node(SkyNode::default(), render_graph::SKY_NODE);
        render_graph.insert_node(PbrNode::default(), render_graph::PBR_NODE);

        render_graph
            .insert_slot_edge(
                ike_render::render_graph::INPUT,
                ViewInputNode::TARGET,
                render_graph::PBR_NODE,
                PbrNode::TARGET,
            )
            .unwrap();

        render_graph
            .insert_slot_edge(
                ike_render::render_graph::DEPTH,
                DepthTextureNode::DEPTH,
                render_graph::PBR_NODE,
                PbrNode::DEPTH,
            )
            .unwrap();

        render_graph
            .insert_slot_edge(
                ike_render::render_graph::INPUT,
                ViewInputNode::CAMERA,
                render_graph::PBR_NODE,
                PbrNode::CAMERA,
            )
            .unwrap();

        render_graph
            .insert_slot_edge(
                ike_render::render_graph::INPUT,
                ViewInputNode::TARGET,
                render_graph::SKY_NODE,
                PbrNode::TARGET,
            )
            .unwrap();

        render_graph
            .insert_slot_edge(
                ike_render::render_graph::INPUT,
                ViewInputNode::CAMERA,
                render_graph::SKY_NODE,
                PbrNode::CAMERA,
            )
            .unwrap();

        render_graph
            .insert_node_edge(render_graph::SKY_NODE, render_graph::PBR_NODE)
            .unwrap();

        render_graph
            .insert_node_edge(
                render_graph::PBR_NODE,
                ike_debug_line::render_graph::DEBUG_LINE_NODE,
            )
            .unwrap();
    }
}
