mod component;
mod function;
mod node;
mod stage;

pub use component::*;
pub use function::*;
pub use node::*;
pub use stage::*;

pub use ike_macro::node;

use ike_app::{App, CoreStage, Plugin};
use ike_ecs::{CommandQueue, Commands, Events, IntoExclusiveSystem, Resource, StageLabel, World};

pub fn node_stage_fn(name: &'static str) -> impl FnMut(&mut World) {
    move |world| {
        let command_queue = CommandQueue::new();
        let commands = Commands::new(world, &command_queue);

        {
            let mut node_stages = world.resource_mut::<NodeStages>();

            if let Some(stage) = node_stages.get_stage_mut(name) {
                stage.run(world, &commands);
            }
        }

        command_queue.apply(world);
    }
}

pub fn node_event_fn<T: Resource>(name: &'static str) -> impl FnMut(&mut World) {
    let mut last_event_count = 0;

    move |world| {
        let command_queue = CommandQueue::new();
        let commands = Commands::new(world, &command_queue);

        {
            let mut node_stages = world.resource_mut::<NodeStages>();
            let events = world.resource::<Events<T>>();

            for event in events.read_from(&mut last_event_count).iter() {
                if let Some(stage) = node_stages.get_event_stage_mut::<T>(name) {
                    stage.run(world, &commands, event);
                }
            }
        }

        command_queue.apply(world);
    }
}

pub trait AddNode {
    fn register_node<T: NodeComponent>(&mut self) -> &mut Self;

    fn add_node_stage(&mut self, name: &'static str, stage: impl StageLabel) -> &mut Self;

    fn add_node_event<T: Resource>(&mut self, name: &'static str) -> &mut Self;
}

impl AddNode for App {
    fn register_node<T: NodeComponent>(&mut self) -> &mut Self {
        T::stages(&mut self.world.resource_mut::<NodeStages>());

        self
    }

    fn add_node_stage(&mut self, name: &'static str, stage: impl StageLabel) -> &mut Self {
        self.add_system_to_stage(node_stage_fn(name).exclusive_system(), stage);

        self
    }

    fn add_node_event<T: Resource>(&mut self, name: &'static str) -> &mut Self {
        self.add_system_to_stage(
            node_event_fn::<T>(name).exclusive_system(),
            CoreStage::PreUpdate,
        );

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

            app.add_system_to_stage(
                collision_node_event_system().exclusive_system(),
                CoreStage::PreUpdate,
            );
        }

        #[cfg(feature = "render")]
        {
            app.add_node_stage("pre_render", ike_render::RenderStage::PreRender);
            app.add_node_stage("render", ike_render::RenderStage::Render);
            app.add_node_stage("post_render", ike_render::RenderStage::PostRender);
        }
    }
}

#[cfg(feature = "physics")]
fn collision_node_event_system() -> impl FnMut(&mut World) {
    use ike_ecs::Entity;
    use ike_physics::Collision;

    let mut last_event_count = 0;

    move |world| {
        let command_queue = CommandQueue::new();
        let commands = Commands::new(world, &command_queue);

        {
            let mut node_stages = world.resource_mut::<NodeStages>();
            let events = world.resource::<Events<Collision>>();
            let entities = world.query::<Entity>().unwrap();

            for event in events.read_from(&mut last_event_count).iter() {
                match event {
                    Collision::Started(a, b) => {
                        if let Some(stage) =
                            node_stages.get_event_stage_mut::<Entity>("collision_started")
                        {
                            if entities.contains(a) && entities.contains(b) {
                                stage.run_single(*a, world, &commands, b);
                                stage.run_single(*b, world, &commands, a);
                            }
                        }
                    }
                    Collision::Stopped(a, b) => {
                        if let Some(stage) =
                            node_stages.get_event_stage_mut::<Entity>("collision_stopped")
                        {
                            stage.run_single(*a, world, &commands, b);
                            stage.run_single(*b, world, &commands, a);
                        }
                    }
                }
            }
        }

        command_queue.apply(world);
    }
}
