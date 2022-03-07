use crate::{RenderGraphError, RenderGraphResult, RenderNodeId};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edge {
    SlotEdge {
        output_node: RenderNodeId,
        output_index: usize,
        input_node: RenderNodeId,
        input_index: usize,
    },
    NodeEdge {
        output_node: RenderNodeId,
        input_node: RenderNodeId,
    },
}

impl Edge {
    pub fn input_node(&self) -> RenderNodeId {
        match self {
            &Self::SlotEdge { input_node, .. } => input_node,
            &Self::NodeEdge { input_node, .. } => input_node,
        }
    }

    pub fn output_node(&self) -> RenderNodeId {
        match self {
            &Self::SlotEdge { output_node, .. } => output_node,
            &Self::NodeEdge { output_node, .. } => output_node,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Edges {
    edges: Vec<Edge>,
}

impl Edges {
    pub const fn new() -> Self {
        Self { edges: Vec::new() }
    }

    pub(crate) fn add_edge(&mut self, edge: Edge) -> RenderGraphResult<()> {
        if self.contains(&edge) {
            return Err(RenderGraphError::EdgeAlreadyExists(edge));
        }

        self.edges.push(edge);

        Ok(())
    }

    pub(crate) fn remove_edge(&mut self, remove: Edge) -> RenderGraphResult<()> {
        if !self.contains(&remove) {
            return Err(RenderGraphError::InvalidEdge(remove));
        }

        self.edges.retain(|edge| *edge != remove);

        Ok(())
    }

    pub fn contains_input_slot(&self, slot_index: usize) -> bool {
        for edge in self.iter() {
            match edge {
                Edge::SlotEdge { input_index, .. } => {
                    if *input_index == slot_index {
                        return true;
                    }
                }
                Edge::NodeEdge { .. } => {}
            }
        }

        false
    }

    pub fn contains_output_slot(&self, slot_index: usize) -> bool {
        for edge in self.iter() {
            match edge {
                Edge::SlotEdge { output_index, .. } => {
                    if *output_index == slot_index {
                        return true;
                    }
                }
                Edge::NodeEdge { .. } => {}
            }
        }

        false
    }

    pub fn contains(&self, edge: &Edge) -> bool {
        self.edges.contains(edge)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Edge> {
        self.edges.iter()
    }
}

impl IntoIterator for Edges {
    type Item = Edge;
    type IntoIter = std::vec::IntoIter<Edge>;

    fn into_iter(self) -> Self::IntoIter {
        self.edges.into_iter()
    }
}
