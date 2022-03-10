mod component;
mod node;
mod node_stage;

pub use component::*;
use ike_ecs::{IntoExclusiveSystem, StageLabel};
pub use node::*;
pub use node_stage::*;

pub use ike_macro::node;

use ike_app::{App, CoreStage, Plugin};

pub trait AddNode {
    fn register_node<T: NodeComponent>(&mut self) -> &mut Self;

    fn add_node_stage(&mut self, name: &'static str, stage: impl StageLabel) -> &mut Self;
}

impl AddNode for App {
    fn register_node<T: NodeComponent>(&mut self) -> &mut Self {
        let mut node_stages = self.world.resource_mut::<NodeStages>();
        node_stages.add_node::<T>();
        drop(node_stages);

        self
    }

    fn add_node_stage(&mut self, name: &'static str, stage: impl StageLabel) -> &mut Self {
        self.add_system_to_stage(node_stage_fn(name).exclusive_system(), stage);

        self
    }
}

#[derive(Default)]
pub struct NodePlugin;

impl Plugin for NodePlugin {
    fn build(self, app: &mut App) {
        app.init_resource::<NodeStages>();

        app.add_node_stage("start", CoreStage::Start);
        app.add_node_stage("pre_update", CoreStage::PreUpdate);
        app.add_node_stage("update", CoreStage::Update);
        app.add_node_stage("post_update", CoreStage::PostUpdate);
        app.add_node_stage("end", CoreStage::End);

        #[cfg(feature = "physics")]
        {
            app.add_node_stage("pre_physics", ike_physics::PhysicsStage::PrePhysics);
            app.add_node_stage("post_physics", ike_physics::PhysicsStage::PostPhysics);
        }

        #[cfg(feature = "render")]
        {
            app.add_node_stage("pre_render", ike_render::RenderStage::PreRender);
            app.add_node_stage("render", ike_render::RenderStage::Render);
            app.add_node_stage("post_render", ike_render::RenderStage::PostRender);
        }
    }
}
