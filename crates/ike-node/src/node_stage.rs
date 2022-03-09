use std::collections::HashMap;

use ike_ecs::{CommandQueue, Commands, Entity, With, World};

use crate::{Node, NodeComponent};

pub fn node_stage_fn(stage_name: &'static str) -> impl FnMut(&mut World) {
    move |world: &mut World| {
        let command_queue = CommandQueue::new();
        let commands = Commands::new(world, &command_queue);

        {
            let node_stages = world.resource::<NodeStages>();

            if let Some(stage) = node_stages.stages.get(stage_name) {
                for node_fn in stage {
                    node_fn(world, &commands);
                }
            }
        }

        command_queue.apply(world);
    }
}

#[derive(Default)]
pub struct NodeStages {
    stages: HashMap<&'static str, Vec<Box<dyn Fn(&World, &Commands) + Send + Sync>>>,
}

impl NodeStages {
    pub fn add_node<T: NodeComponent>(&mut self) {
        for node_fn in T::stages() {
            self.stages.entry(node_fn.name).or_default().push(Box::new(
                |world: &World, commands: &Commands| {
                    let query = world.query_filter::<Entity, With<T>>().unwrap();

                    for entity in query.iter() {
                        let node = Node::new(entity, world, commands);
                        let component = node.component_mut::<T>();
                        (node_fn.func)(component, node);
                    }
                },
            ));
        }
    }
}
