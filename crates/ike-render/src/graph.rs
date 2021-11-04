use std::collections::{HashMap, HashSet};

use ike_core::World;

use crate::{render_device, render_queue, EdgeSlotInfo, NodeEdge, NodeInput};

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
    fn update(&mut self, _world: &mut World) {}

    fn run(
        &mut self,
        encoder: &mut crate::wgpu::CommandEncoder,
        world: &World,
        input: &NodeInput,
        output: &mut NodeEdge,
    ) -> Result<(), GraphError>;
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum GraphError {
    #[error("failed to find node '{0}'")]
    NodeNotFound(String),

    #[error("edge not connected '{0}' to '{1}'")]
    EdgeNotConnected(String, String),

    #[error("failed to find slot '{0}'")]
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
    #[allow(unused)]
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
    edges: HashMap<String, HashSet<String>>,
    slots: HashMap<String, HashMap<String, SlotConnection>>,
    nodes: HashMap<String, NodeContainer>,
    end: HashSet<String>,
    stages: Vec<HashSet<String>>,
}

impl RenderGraph {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    fn get_node_container(&self, name: &str) -> Result<&NodeContainer, GraphError> {
        if let Some(node) = self.nodes.get(name) {
            Ok(node)
        } else {
            Err(GraphError::NodeNotFound(String::from(name)))
        }
    }

    #[inline]
    fn get_node_container_mut(&mut self, name: &str) -> Result<&mut NodeContainer, GraphError> {
        if let Some(node) = self.nodes.get_mut(name) {
            Ok(node)
        } else {
            Err(GraphError::NodeNotFound(String::from(name)))
        }
    }

    #[inline]
    pub fn has_node(&self, name: impl AsRef<str>) -> bool {
        self.nodes.contains_key(name.as_ref())
    }

    #[inline]
    pub fn insert_node<T: RenderNode, U: Into<String>>(&mut self, render_node: T, name: U) {
        let name = name.into();

        self.nodes
            .insert(name.clone(), NodeContainer::new(render_node));

        self.end.insert(name);

        self.calculate_stages();
    }

    #[inline]
    pub fn insert_node_edge(
        &mut self,
        from: impl AsRef<str>,
        to: impl AsRef<str>,
    ) -> Result<(), GraphError> {
        let from = from.as_ref();
        let to = to.as_ref();

        self.get_node_container(from)?;
        self.get_node_container(to)?;

        if !self.edges.contains_key(to) {
            self.edges.insert(String::from(to), HashSet::new());
        }

        self.edges.get_mut(to).unwrap().insert(String::from(from));

        self.end.remove(from);

        self.calculate_stages();

        Ok(())
    }

    #[inline]
    pub fn insert_slot_edge(
        &mut self,
        from: impl AsRef<str>,
        output: impl AsRef<str>,
        to: impl AsRef<str>,
        input: impl AsRef<str>,
    ) -> Result<(), GraphError> {
        if !self.slots.contains_key(to.as_ref()) {
            self.slots.insert(String::from(to.as_ref()), HashMap::new());
        }

        let slot_edges = self.slots.get_mut(to.as_ref()).unwrap();

        slot_edges.insert(
            String::from(input.as_ref()),
            SlotConnection {
                node: String::from(from.as_ref()),
                slot: String::from(output.as_ref()),
            },
        );

        self.insert_node_edge(from, to)?;

        self.calculate_stages();

        Ok(())
    }

    #[inline]
    pub fn get_output(&self, name: impl AsRef<str>) -> Result<&NodeEdge, GraphError> {
        Ok(&self.get_node_container(name.as_ref())?.output)
    }

    #[inline]
    pub fn get_output_mut(&mut self, name: impl AsRef<str>) -> Result<&mut NodeEdge, GraphError> {
        Ok(&mut self.get_node_container_mut(name.as_ref())?.output)
    }

    #[inline]
    pub fn remove_node(&mut self, name: impl AsRef<str>) -> Result<(), GraphError> {
        let name = name.as_ref();

        self.edges.remove(name);
        self.slots.remove(name);

        if self.nodes.remove(name).is_some() {
            self.calculate_stages();

            Ok(())
        } else {
            Err(GraphError::NodeNotFound(String::from(name)))
        }
    }

    #[inline]
    pub fn remove_edge(
        &mut self,
        from: impl AsRef<str>,
        to: impl AsRef<str>,
    ) -> Result<(), GraphError> {
        let from = from.as_ref();
        let to = to.as_ref();

        self.get_node_container(from)?;

        if let Some(edges) = self.edges.get_mut(from) {
            if !edges.remove(to) {
                return Err(GraphError::EdgeNotConnected(
                    String::from(from),
                    String::from(to),
                ));
            }
        }

        self.calculate_stages();

        Ok(())
    }

    #[inline]
    pub fn update(&mut self, world: &mut World) {
        for node in self.nodes.values_mut() {
            node.node.update(world);
        }
    }

    #[inline]
    pub fn calculate_stages(&mut self) {
        self.stages.clear();

        let mut stage = self.end.clone();

        while !stage.is_empty() {
            self.stages.push(stage.clone());

            for name in std::mem::replace(&mut stage, HashSet::new()) {
                for edge in &self.edges[&name] {
                    stage.insert(edge.clone());
                }
            }
        }
    }

    #[inline]
    pub fn run(&mut self, world: &World) -> Result<(), GraphError> {
        let mut encoder = render_device().create_command_encoder(&Default::default());

        for stage in self.stages.iter().rev() {
            for name in stage {
                let mut node = self.nodes.remove(name).unwrap();

                let mut input = NodeInput::default();

                if let Some(slots) = self.slots.get(name) {
                    for (slot_name, i) in slots {
                        let slot = self.nodes[&i.node].output.get_slot(&i.slot).unwrap();

                        input.slots.insert(slot_name.clone(), &slot);
                    }
                }

                node.node
                    .run(&mut encoder, world, &input, &mut node.output)?;

                node.output.slots_set()?;

                self.nodes.insert(name.clone(), node);
            }
        }

        render_queue().submit_once(encoder.finish());

        Ok(())
    }
}
