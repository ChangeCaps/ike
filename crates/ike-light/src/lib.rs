mod directional_light;
mod light_bindings;
mod light_node;
mod point_light;
mod spot_light;

pub use directional_light::*;
use ike_render::RenderGraph;
pub use light_bindings::*;
pub use light_node::*;
pub use point_light::*;
pub use spot_light::*;

pub mod node {
    pub const LIGHT_NODE: &str = "light_node";
}

pub const MAX_DIRECTIONAL_LIGHTS: u32 = 8;
pub const DIRECTIONAL_LIGHT_SHADOW_MAP_SIZE: u32 = 4096;

use ike_app::{App, Plugin};

#[derive(Default)]
pub struct LightPlugin;

impl Plugin for LightPlugin {
    fn build(self, app: &mut App) {
        app.init_resource::<LightBindings>();
        app.init_resource::<LightPipeline>();

        let mut render_graph = app.world.resource_mut::<RenderGraph>();

        render_graph.add_node(node::LIGHT_NODE, LightNode::default());

        render_graph
            .add_node_edge(node::LIGHT_NODE, ike_render::node::DEPENDENCIES)
            .unwrap();
    }
}
