use std::{
    any::{type_name, Any},
    borrow::Cow,
    collections::HashMap,
};

use ike_ecs::World;

use crate::{
    Edge, RenderContext, RenderNode, RenderNodeId, RenderNodeLabel, RenderNodeState, SlotInfo,
    SlotInfos, SlotLabel, SlotType, SlotValue,
};

pub struct RenderGraphContext<'a> {
    pub(crate) inputs: Vec<&'a SlotValue>,
    pub(crate) input_slots: &'a SlotInfos,
    pub(crate) outputs: &'a mut [Option<SlotValue>],
    pub(crate) output_slots: &'a SlotInfos,
}

impl<'a> RenderGraphContext<'a> {
    pub fn get_input<T: Any>(&self, label: &'static str) -> RenderGraphResult<&T> {
        let index = self
            .input_slots
            .get_slot_index(label)
            .ok_or(RenderGraphError::InvalidInputNodeSlot(label.into()))?;

        self.inputs[index]
            .downcast()
            .ok_or(RenderGraphError::MismatchedValueTypes {
                slot_ty: self.inputs[index].slot_type().type_name(),
                ty: type_name::<T>(),
            })
    }

    pub fn set_output<T: Any + Send + Sync>(
        &mut self,
        value: T,
        label: impl Into<SlotLabel>,
    ) -> RenderGraphResult<()> {
        let label = label.into();

        let index = self
            .output_slots
            .get_slot_index(&label)
            .ok_or(RenderGraphError::InvalidOutputNodeSlot(label.into()))?;

        if *self.output_slots.get(index).unwrap().ty() == SlotType::new::<T>() {
            self.outputs[index] = Some(SlotValue::new(value));
        } else {
            return Err(RenderGraphError::MismatchedValueTypes {
                slot_ty: self.output_slots.get(index).unwrap().ty().type_name(),
                ty: type_name::<T>(),
            });
        }

        Ok(())
    }

    pub fn get_output<T: Any>(&self, label: &'static str) -> RenderGraphResult<Option<&T>> {
        let index = self
            .output_slots
            .get_slot_index(label)
            .ok_or(RenderGraphError::InvalidOutputNodeSlot(label.into()))?;

        if let Some(ref output) = self.outputs[index] {
            Ok(Some(output.downcast().ok_or(
                RenderGraphError::MismatchedValueTypes {
                    slot_ty: self.output_slots.get(index).unwrap().ty().type_name(),
                    ty: type_name::<T>(),
                },
            )?))
        } else {
            Ok(None)
        }
    }

    pub fn get_output_mut<T: Any>(
        &mut self,
        label: &'static str,
    ) -> RenderGraphResult<Option<&mut T>> {
        let index = self
            .output_slots
            .get_slot_index(label)
            .ok_or(RenderGraphError::InvalidOutputNodeSlot(label.into()))?;

        if let Some(ref mut output) = self.outputs[index] {
            Ok(Some(output.downcast_mut().ok_or(
                RenderGraphError::MismatchedValueTypes {
                    slot_ty: self.output_slots.get(index).unwrap().ty().type_name(),
                    ty: type_name::<T>(),
                },
            )?))
        } else {
            Ok(None)
        }
    }
}

#[derive(Default)]
pub struct RenderGraph {
    pub(crate) nodes: HashMap<RenderNodeId, RenderNodeState>,
    pub(crate) node_names: HashMap<Cow<'static, str>, RenderNodeId>,
    pub(crate) input_node: Option<RenderNodeId>,
}

impl RenderGraph {
    pub const INPUT_NODE_NAME: &'static str = "GraphInputNode";

    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_node_id(
        &self,
        label: impl Into<RenderNodeLabel>,
    ) -> RenderGraphResult<RenderNodeId> {
        let label = label.into();

        match &label {
            RenderNodeLabel::Id(id) => Ok(*id),
            RenderNodeLabel::Label(name) => self
                .node_names
                .get(name)
                .cloned()
                .ok_or(RenderGraphError::InvalidNode(label)),
        }
    }

    pub fn set_input(&mut self, inputs: Vec<SlotInfo>) -> RenderNodeId {
        let id = self.add_node(Self::INPUT_NODE_NAME, GraphInputNode);
        let state = self.get_node_state_mut(&id).unwrap();
        state.output_slots = SlotInfos::new(inputs);
        state.outputs.resize_with(state.output_slots.len(), || None);
        self.input_node = Some(id);
        id
    }

    pub fn input_node(&self) -> Option<RenderNodeId> {
        self.input_node
    }

    pub fn update(&mut self, world: &mut World) {
        for state in self.nodes.values_mut() {
            state.node.update(world);
        }
    }

    pub fn add_node<T: RenderNode>(
        &mut self,
        name: impl Into<Cow<'static, str>>,
        node: T,
    ) -> RenderNodeId {
        let id = RenderNodeId::new();
        let name = name.into();
        let mut state = RenderNodeState::new(id, node);
        state.name = Some(name.clone());
        self.nodes.insert(id, state);
        self.node_names.insert(name, id);
        id
    }

    pub fn add_slot_edge(
        &mut self,
        output_node: impl Into<RenderNodeLabel>,
        output_slot: impl Into<SlotLabel>,
        input_node: impl Into<RenderNodeLabel>,
        input_slot: impl Into<SlotLabel>,
    ) -> RenderGraphResult<()> {
        let output_slot = output_slot.into();
        let input_slot = input_slot.into();
        let output_node_id = self.get_node_id(output_node)?;
        let input_node_id = self.get_node_id(input_node)?;

        let output_index = self
            .get_node_state(output_node_id)?
            .output_slots
            .get_slot_index(output_slot.clone())
            .ok_or(RenderGraphError::InvalidOutputNodeSlot(output_slot))?;

        let input_index = self
            .get_node_state(input_node_id)?
            .input_slots
            .get_slot_index(input_slot.clone())
            .ok_or(RenderGraphError::InvalidInputNodeSlot(input_slot))?;

        let edge = Edge::SlotEdge {
            output_node: output_node_id,
            output_index,
            input_node: input_node_id,
            input_index,
        };

        self.validate_edge(&edge)?;

        let output_node = self.get_node_state_mut(output_node_id)?;
        output_node.output_edges.add_edge(edge)?;

        let input_node = self.get_node_state_mut(input_node_id)?;
        input_node.input_edges.add_edge(edge)?;

        Ok(())
    }

    pub fn add_node_edge(
        &mut self,
        output: impl Into<RenderNodeLabel>,
        input: impl Into<RenderNodeLabel>,
    ) -> RenderGraphResult<()> {
        let output_node_id = self.get_node_id(output)?;
        let input_node_id = self.get_node_id(input)?;

        let edge = Edge::NodeEdge {
            output_node: output_node_id,
            input_node: input_node_id,
        };

        self.validate_edge(&edge)?;

        let output_node = self.get_node_state_mut(output_node_id)?;
        output_node.output_edges.add_edge(edge)?;

        let input_node = self.get_node_state_mut(input_node_id)?;
        input_node.input_edges.add_edge(edge)?;

        Ok(())
    }

    pub fn remove_slot_edge(
        &mut self,
        output_node: impl Into<RenderNodeLabel>,
        output_slot: impl Into<SlotLabel>,
        input_node: impl Into<RenderNodeLabel>,
        input_slot: impl Into<SlotLabel>,
    ) -> RenderGraphResult<()> {
        let output_slot = output_slot.into();
        let input_slot = input_slot.into();
        let output_node_id = self.get_node_id(output_node)?;
        let input_node_id = self.get_node_id(input_node)?;

        let output_index = self
            .get_node_state(output_node_id)?
            .output_slots
            .get_slot_index(output_slot.clone())
            .ok_or(RenderGraphError::InvalidOutputNodeSlot(output_slot))?;

        let input_index = self
            .get_node_state(input_node_id)?
            .input_slots
            .get_slot_index(input_slot.clone())
            .ok_or(RenderGraphError::InvalidInputNodeSlot(input_slot))?;

        let edge = Edge::SlotEdge {
            output_node: output_node_id,
            output_index,
            input_node: input_node_id,
            input_index,
        };

        let output_node = self.get_node_state_mut(output_node_id)?;
        output_node.output_edges.remove_edge(edge)?;

        let input_node = self.get_node_state_mut(input_node_id)?;
        input_node.input_edges.remove_edge(edge)?;

        Ok(())
    }

    pub fn remove_node_edge(
        &mut self,
        output: impl Into<RenderNodeLabel>,
        input: impl Into<RenderNodeLabel>,
    ) -> RenderGraphResult<()> {
        let output_node_id = self.get_node_id(output)?;
        let input_node_id = self.get_node_id(input)?;

        let edge = Edge::NodeEdge {
            output_node: output_node_id,
            input_node: input_node_id,
        };

        let output_node = self.get_node_state_mut(output_node_id)?;
        output_node.output_edges.remove_edge(edge)?;

        let input_node = self.get_node_state_mut(input_node_id)?;
        input_node.input_edges.remove_edge(edge)?;

        Ok(())
    }

    pub fn get_node_state(
        &self,
        label: impl Into<RenderNodeLabel>,
    ) -> RenderGraphResult<&RenderNodeState> {
        let label = label.into();
        let id = self.get_node_id(&label)?;
        self.nodes
            .get(&id)
            .ok_or(RenderGraphError::InvalidNode(label))
    }

    pub fn get_node_state_mut(
        &mut self,
        label: impl Into<RenderNodeLabel>,
    ) -> RenderGraphResult<&mut RenderNodeState> {
        let label = label.into();
        let id = self.get_node_id(&label)?;
        self.nodes
            .get_mut(&id)
            .ok_or(RenderGraphError::InvalidNode(label))
    }

    pub fn validate_edge(&self, edge: &Edge) -> RenderGraphResult<()> {
        if self.has_edge(edge) {
            return Err(RenderGraphError::EdgeAlreadyExists(*edge));
        }

        match edge {
            &Edge::SlotEdge {
                output_node,
                output_index,
                input_node,
                input_index,
            } => {
                let output_node_state = self.get_node_state(output_node)?;
                let input_node_state = self.get_node_state(input_node)?;

                let output_slot = output_node_state.output_slots.get(output_index).ok_or(
                    RenderGraphError::InvalidOutputNodeSlot(SlotLabel::Index(output_index)),
                )?;

                let input_slot = input_node_state.input_slots.get(input_index).ok_or(
                    RenderGraphError::InvalidInputNodeSlot(SlotLabel::Index(input_index)),
                )?;

                if input_node_state
                    .input_edges
                    .contains_output_slot(input_index)
                {
                    return Err(RenderGraphError::NodeInputSlotAlreadyOccupied {
                        node: input_node,
                        input_index,
                        occupied_by: output_node,
                    });
                }

                if output_slot.ty() != input_slot.ty() {
                    return Err(RenderGraphError::MismatchedNodeSlots {
                        input_node,
                        input_index,
                        output_node,
                        output_index,
                    });
                }
            }
            Edge::NodeEdge { .. } => {}
        }

        Ok(())
    }

    pub fn has_edge(&self, edge: &Edge) -> bool {
        let input_node_state = self.get_node_state(edge.input_node());
        let output_node_state = self.get_node_state(edge.output_node());

        if let (Ok(input_node_state), Ok(output_node_state)) = (input_node_state, output_node_state)
        {
            if input_node_state.input_edges.contains(edge)
                && output_node_state.output_edges.contains(edge)
            {
                return true;
            }
        }

        false
    }
}

#[derive(Default)]
pub struct GraphInputNode;

impl RenderNode for GraphInputNode {
    fn run(
        &mut self,
        _graph_context: &mut RenderGraphContext<'_>,
        _render_context: &mut RenderContext,
        _world: &World,
    ) -> RenderGraphResult<()> {
        Ok(())
    }
}

#[derive(Default)]
pub struct EmptyNode;

impl RenderNode for EmptyNode {
    fn run(
        &mut self,
        _graph_context: &mut RenderGraphContext<'_>,
        _render_context: &mut RenderContext,
        _world: &World,
    ) -> RenderGraphResult<()> {
        Ok(())
    }
}

pub type RenderGraphResult<T> = Result<T, RenderGraphError>;

#[derive(thiserror::Error, Debug)]
pub enum RenderGraphError {
    #[error("node does not exist {0:?}")]
    InvalidNode(RenderNodeLabel),
    #[error("edge does not exist")]
    InvalidEdge(Edge),
    #[error("output slot does not exist {0:?}")]
    InvalidOutputNodeSlot(SlotLabel),
    #[error("input slot does not exist {0:?}")]
    InvalidInputNodeSlot(SlotLabel),
    #[error("attempted to get slot value '{slot_ty}' as '{ty}'")]
    MismatchedValueTypes {
        slot_ty: &'static str,
        ty: &'static str,
    },
    #[error("attempted to connect output node slot to invalid input node slot")]
    MismatchedNodeSlots {
        input_node: RenderNodeId,
        input_index: usize,
        output_node: RenderNodeId,
        output_index: usize,
    },
    #[error("edge {0:?} already exists")]
    EdgeAlreadyExists(Edge),
    #[error("node {0:?} has an unconnected input slot {1:?}")]
    UnconnectedInputNodeSlot(RenderNodeId, SlotLabel),
    #[error("node {0:?} has an unconnected output slot {1:?}")]
    UnconnectedOutputNodeSlot(RenderNodeId, SlotLabel),
    #[error("node input slot already occupied")]
    NodeInputSlotAlreadyOccupied {
        node: RenderNodeId,
        input_index: usize,
        occupied_by: RenderNodeId,
    },
    #[error("no input node")]
    RunWithoutInputNode,
    #[error("invalid input count")]
    InvalidInputCount,
}
