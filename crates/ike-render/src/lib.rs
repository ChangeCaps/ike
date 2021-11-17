mod buffer;
mod camera;
mod color;
mod cube_texture;
mod edge;
mod graph;
mod mesh;
mod node;
mod render_ctx;
mod render_texture;
mod shader;
mod shape;
mod system;
mod texture;

pub use buffer::*;
pub use camera::*;
pub use color::*;
pub use cube_texture::*;
pub use edge::*;
pub use graph::*;
pub use mesh::*;
pub use node::*;
pub use render_ctx::*;
pub use render_texture::*;
pub use shader::*;
pub use shape::*;
pub use system::*;
pub use texture::*;

pub use ike_wgpu as wgpu;

pub mod render_graph {
    pub const INPUT: &str = "input";
    pub const DEPTH: &str = "depth";
}

use ike_assets::AssetAppBuilderExt;
use ike_core::*;
use ike_reflect::ReflectAppBuilderExt;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    #[inline]
    fn build(self, app: &mut AppBuilder) {
        let mut render_graph = RenderGraph::new();

        render_graph.insert_node(ViewInputNode, render_graph::INPUT);
        render_graph.insert_node(DepthTextureNode::default(), render_graph::DEPTH);

        render_graph
            .insert_slot_edge(
                render_graph::INPUT,
                ViewInputNode::TARGET,
                render_graph::DEPTH,
                DepthTextureNode::TARGET,
            )
            .unwrap();

        app.insert_resource(render_graph);
        app.add_asset::<Mesh>();
        app.add_asset::<Texture>();
        app.add_asset::<Environment>();
        app.add_system_to_stage(render_graph_update_system.system(), stage::PRE_RENDER);
        app.add_system_to_stage(camera_aspect_system.system(), stage::PRE_RENDER);
        app.add_system_to_stage(render_system.system(), stage::RENDER);
        app.register::<PerspectiveProjection>();
        app.register::<OrthographicProjection>();
        app.register::<MainCamera>();
    }
}
