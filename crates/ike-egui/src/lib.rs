mod node;
mod resource;

pub use node::*;
pub use resource::*;

use ike_render::*;
use ike_core::*;

pub mod render_graph {
    pub const EGUI_NODE: &str = "egui_node";
}

#[derive(Default)]
pub struct EguiPlugin;

impl Plugin for EguiPlugin {
    #[inline]
    fn build(self, app: &mut AppBuilder) {
        app.world_mut().init_resource::<egui::CtxRef>();
        app.world_mut().init_resource::<EguiTextures>();

        let mut render_graph = app.world().write_resource::<RenderGraph>().unwrap();

        //if render_graph
    }
}
