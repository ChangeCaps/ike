mod node;
mod resource;
mod system;

pub use node::*;
pub use resource::*;
pub use system::*;

pub use egui;

use ike_core::*;
use ike_render::*;

pub mod render_graph {
    pub const EGUI_NODE: &str = "egui_node";
}

#[derive(Default)]
pub struct EguiPlugin;

impl Plugin for EguiPlugin {
    #[inline]
    fn build(self, app: &mut AppBuilder) {
        let mut raw_input = egui::RawInput::default();
        let mut ctx = egui::CtxRef::default();

        ctx.begin_frame(raw_input.take());

        app.add_system_to_stage(egui_input_system.system(), stage::PRE_UPDATE);
        app.world_mut().insert_resource(ctx);
        app.world_mut().insert_resource(raw_input);
        app.world_mut().init_resource::<EguiTextures>();

        let mut render_graph = app.world().write_resource::<RenderGraph>().unwrap();

        if render_graph.has_node(ike_pbr::render_graph::PBR_NODE) {
            render_graph.insert_node(EguiNode::default(), render_graph::EGUI_NODE);

            render_graph
                .insert_node_edge(ike_pbr::render_graph::PBR_NODE, render_graph::EGUI_NODE)
                .unwrap();

            render_graph
                .insert_slot_edge(
                    ike_render::render_graph::INPUT,
                    ViewInputNode::TARGET,
                    render_graph::EGUI_NODE,
                    EguiNode::TARGET,
                )
                .unwrap();
        } else {
            render_graph.insert_node(EguiNode::clear(Color::TRANSPARENT), render_graph::EGUI_NODE);

            render_graph
                .insert_slot_edge(
                    ike_render::render_graph::INPUT,
                    ViewInputNode::TARGET,
                    render_graph::EGUI_NODE,
                    EguiNode::TARGET,
                )
                .unwrap();
        }
    }
}
