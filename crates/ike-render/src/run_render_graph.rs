use std::collections::VecDeque;

use ike_ecs::World;

use crate::{
    CommandEncoderDescriptor, Edge, RenderContext, RenderDevice, RenderGraph, RenderGraphContext,
    RenderGraphError, RenderGraphResult, RenderNodeId, RenderQueue, SlotValue,
};

impl RenderGraph {
    pub fn run(&mut self, world: &World, input: Vec<SlotValue>) -> RenderGraphResult<()> {
        self.validate_nodes()?;

        let mut render_context = {
            let device = world.resource::<RenderDevice>().clone();
            let queue = world.resource::<RenderQueue>().clone();

            let encoder = device.create_command_encoder(&CommandEncoderDescriptor {
                label: Some("ike_render_graph_command_encoder"),
            });

            RenderContext {
                device,
                queue,
                encoder,
            }
        };

        if let Some(input_node) = self.input_node() {
            let state = self.get_node_state_mut(input_node)?;

            if input.len() != state.output_slots.len() {
                return Err(RenderGraphError::InvalidInputCount);
            }

            for (i, value) in input.into_iter().enumerate() {
                if state.output_slots.get(i).unwrap().ty() == value.slot_type() {
                    state.outputs[i] = Some(value);
                } else {
                    return Err(RenderGraphError::MismatchedNodeSlots {
                        output_node: input_node,
                        output_index: i,
                        input_node,
                        input_index: i,
                    });
                }
            }
        } else {
            return Err(RenderGraphError::RunWithoutInputNode);
        }

        let mut processed = Vec::new();
        let mut node_queue: VecDeque<RenderNodeId> = self
            .nodes
            .values()
            .filter_map(|state| {
                if state.input_slots.is_empty() {
                    Some(state.id)
                } else {
                    None
                }
            })
            .collect();

        'handle_node: while let Some(node) = node_queue.pop_front() {
            let mut state = self.nodes.remove(&node).unwrap();

            let mut inputs = Vec::new();

            for edge in state.input_edges.iter() {
                if !processed.contains(&edge.output_node()) {
                    continue 'handle_node;
                }

                match edge {
                    Edge::SlotEdge {
                        output_index,
                        output_node,
                        ..
                    } => {
                        let output_state = self.get_node_state(output_node)?;

                        inputs.push((
                            output_index,
                            output_state.outputs[*output_index].as_ref().unwrap(),
                        ));
                    }
                    Edge::NodeEdge { .. } => {}
                }
            }

            inputs.sort_by_key(|(index, _)| **index);

            let mut graph_context = RenderGraphContext {
                inputs: inputs
                    .into_iter()
                    .map(|(_, slot_value)| slot_value)
                    .collect(),
                input_slots: &state.input_slots,
                outputs: &mut state.outputs,
                output_slots: &state.output_slots,
            };

            state
                .node
                .run(&mut graph_context, &mut render_context, world)?;

            for edge in state.output_edges.iter() {
                if !node_queue.contains(&edge.input_node()) {
                    node_queue.push_back(edge.input_node());
                }
            }

            self.nodes.insert(node, state);

            processed.push(node);
        }

        render_context
            .queue
            .submit_one(render_context.encoder.finish());

        Ok(())
    }
}
