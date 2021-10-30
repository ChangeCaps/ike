use std::collections::HashMap;

use crate::{EdgeSlotInfo, NodeEdge};

pub trait RenderNode: Send + Sync + 'static {
    #[inline]
    fn input(&self) -> Vec<EdgeSlotInfo> {
        Vec::new()
    }

    #[inline]
    fn output(&self) -> Vec<EdgeSlotInfo> {
        Vec::new()
    }

    #[inline]
    fn update(&mut self) {}

    fn run(
        &mut self,
        encoder: &mut ike_wgpu::CommandEncoder,
        input: &NodeEdge,
        output: &mut NodeEdge,
    ) -> Result<(), GraphError>;
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum GraphError {
    #[error("failed to find node '{0}'")]
    SlotNotFound(String),

    #[error("set wrong type, expected '{expected}' found '{found}'")]
    SetWrongType {
        found: &'static str,
        expected: &'static str,
    },

    #[error("get wrong type, expected '{expected}' found '{found}'")]
    GetWrongType {
        found: &'static str,
        expected: &'static str,
    },

    #[error("slot not set '{0}'")]
    SlotNotSet(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct SlotConnection {
    node: String,
    slot: String,
}

struct NodeContainer {
	input: Vec<EdgeSlotInfo>,
    output: NodeEdge,
    node: Box<dyn RenderNode>,
}

impl NodeContainer {
	#[inline]
	pub fn new<T: RenderNode>(render_node: T) -> Self {
		Self {
			input: render_node.input(),
			output: NodeEdge::from_info(render_node.output()),
			node: Box::new(render_node),
		}
	}
}

#[derive(Default)]
pub struct RenderGraph {
    edges: HashMap<String, Vec<String>>,
    slots: HashMap<SlotConnection, Vec<SlotConnection>>,
    nodes: HashMap<String, NodeContainer>,
}

impl RenderGraph {
	#[inline]
	pub fn insert_node<T: RenderNode, U: Into<String>>(&mut self, render_node: T, name: U) {
		self.nodes.insert(name.into(), NodeContainer::new(render_node));
	}
}