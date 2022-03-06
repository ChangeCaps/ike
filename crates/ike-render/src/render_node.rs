use std::{any::type_name, borrow::Cow};

use ike_ecs::World;
use ike_id::Id;

use crate::{
    CommandEncoder, Edges, RenderDevice, RenderGraphContext, RenderGraphResult, RenderQueue,
    SlotInfo, SlotInfos, SlotValue,
};

pub type RenderNodeId = Id<RenderNodeState>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RenderNodeLabel {
    Id(RenderNodeId),
    Label(Cow<'static, str>),
}

impl From<&RenderNodeLabel> for RenderNodeLabel {
    fn from(label: &RenderNodeLabel) -> Self {
        label.clone()
    }
}

impl From<String> for RenderNodeLabel {
    fn from(label: String) -> Self {
        Self::Label(label.into())
    }
}

impl From<&'static str> for RenderNodeLabel {
    fn from(label: &'static str) -> Self {
        Self::Label(label.into())
    }
}

impl From<&RenderNodeId> for RenderNodeLabel {
    fn from(id: &RenderNodeId) -> Self {
        Self::Id(*id)
    }
}

impl From<RenderNodeId> for RenderNodeLabel {
    fn from(id: RenderNodeId) -> Self {
        Self::Id(id)
    }
}

pub struct RenderContext {
    pub device: RenderDevice,
    pub queue: RenderQueue,
    pub encoder: CommandEncoder,
}

pub trait RenderNode: Send + Sync + 'static {
    fn input() -> Vec<SlotInfo>
    where
        Self: Sized,
    {
        Vec::new()
    }

    fn output() -> Vec<SlotInfo>
    where
        Self: Sized,
    {
        Vec::new()
    }

    fn update(&mut self, _world: &mut World) {}

    fn run(
        &mut self,
        graph_context: &mut RenderGraphContext<'_>,
        render_context: &mut RenderContext,
        world: &World,
    ) -> RenderGraphResult<()>;
}

pub struct RenderNodeState {
    pub id: RenderNodeId,
    pub name: Option<Cow<'static, str>>,
    pub node: Box<dyn RenderNode>,
    pub type_name: &'static str,
    pub input_slots: SlotInfos,
    pub output_slots: SlotInfos,
    pub input_edges: Edges,
    pub output_edges: Edges,
    pub outputs: Vec<Option<SlotValue>>,
}

impl RenderNodeState {
    pub fn new<T: RenderNode>(id: RenderNodeId, node: T) -> Self {
        let output_slots = SlotInfos::new(T::output());
        let mut outputs = Vec::with_capacity(output_slots.len());
        outputs.resize_with(output_slots.len(), || None);

        Self {
            id,
            name: None,
            node: Box::new(node),
            type_name: type_name::<T>(),
            input_slots: SlotInfos::new(T::input()),
            output_slots,
            input_edges: Edges::new(),
            output_edges: Edges::new(),
            outputs,
        }
    }
}
