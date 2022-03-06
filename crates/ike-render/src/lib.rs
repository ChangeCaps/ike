mod camera;
mod color;
mod edge;
pub mod mesh;
mod render_graph;
mod render_node;
mod render_system;
mod resources;
mod run_render_graph;
mod shapes;
mod slot;

pub use camera::*;
pub use color::*;
pub use edge::*;
pub use mesh::{Mesh, MeshBuffers};
pub use render_graph::*;
pub use render_node::*;
pub use render_system::*;
pub use resources::*;
pub use slot::*;
pub use wgpu;

use ike_app::{App, Plugin};
use ike_assets::AddAsset;

pub mod stage {
    pub const PRE_RENDER: &str = "pre_render";
    pub const RENDER: &str = "render";
    pub const POST_RENDER: &str = "post_render";
}

pub mod node {
    pub const SURFACE_TEXTURE: &str = "surface_texture";
    pub const CAMERA: &str = "camera";
}

#[derive(Default)]
pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(self, app: &mut App) {
        let mut render_graph = RenderGraph::default();

        render_graph.set_input(vec![
            SlotInfo::new::<TextureView>(node::SURFACE_TEXTURE),
            SlotInfo::new::<RawCamera>(node::CAMERA),
        ]);

        app.world.insert_resource(render_graph);

        app.add_asset::<Mesh>();
        app.add_asset::<MeshBuffers>();

        app.add_stage_after(stage::PRE_RENDER, ike_app::stage::POST_UPDATE);
        app.add_stage_after(stage::RENDER, stage::PRE_RENDER);
        app.add_stage_after(stage::POST_RENDER, stage::RENDER);

        app.add_exclusive_system_to_stage(render_system, stage::RENDER);
    }
}
