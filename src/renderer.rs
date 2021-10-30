use std::{
    any::{type_name, Any, TypeId},
    borrow::Cow,
    collections::HashMap,
};

use once_cell::sync::OnceCell;

pub struct RenderCtx {
    pub device: ike_wgpu::Device,
    pub queue: ike_wgpu::Queue,
}

static RENDER_CTX: OnceCell<RenderCtx> = OnceCell::new();

#[inline]
pub fn render_device<'a>() -> &'a ike_wgpu::Device {
    &RENDER_CTX.get().expect("RENDER_CTX not set").device
}

#[inline]
pub fn render_queue<'a>() -> &'a ike_wgpu::Queue {
    &RENDER_CTX.get().expect("RENDER_CTX not set").queue
}

#[inline]
pub fn set_render_ctx(render_ctx: RenderCtx) {
    RENDER_CTX
        .set(render_ctx)
        .ok()
        .expect("RENDER_CTX already set");
}

pub struct RenderSurface {
    surface: ike_wgpu::Surface,
    config: ike_wgpu::SurfaceConfiguration,
    updated: bool,
}

impl RenderSurface {
    #[inline]
    pub fn new(surface: ike_wgpu::Surface, config: ike_wgpu::SurfaceConfiguration) -> Self {
        Self {
            surface,
            config,
            updated: true,
        }
    }

    #[inline]
    pub fn config(&self) -> &ike_wgpu::SurfaceConfiguration {
        &self.config
    }

    #[inline]
    pub fn configure(&mut self) -> &mut ike_wgpu::SurfaceConfiguration {
        self.updated = true;

        &mut self.config
    }

    #[inline]
    pub fn surface(&self) -> &ike_wgpu::Surface {
        if self.updated {
            self.surface.configure(render_device(), &self.config);
        }

        &self.surface
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EdgeSlotInfo {
    name: Cow<'static, str>,
    type_name: &'static str,
    type_id: TypeId,
}

impl EdgeSlotInfo {
    #[inline]
    pub fn new<T: Any>(name: &'static str) -> Self {
        Self {
            name: name.into(),
            type_name: type_name::<T>(),
            type_id: TypeId::of::<T>(),
        }
    }

    #[inline]
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    #[inline]
    pub fn ty_name(&self) -> &'static str {
        self.type_name
    }

    #[inline]
    pub fn ty_id(&self) -> TypeId {
        self.type_id
    }
}

pub struct EdgeSlot {
    value: Option<Box<dyn Any>>,
    info: EdgeSlotInfo,
}

impl EdgeSlot {
    #[inline]
    pub fn new(info: EdgeSlotInfo) -> Self {
        Self { value: None, info }
    }
}

pub struct NodeEdge {
    slots: Vec<EdgeSlot>,
}

impl NodeEdge {
    #[inline]
    fn get_slot(&self, name: &str) -> Result<&EdgeSlot, GraphError> {
        self.slots
            .iter()
            .find(|slot| slot.info.name() == name)
            .ok_or_else(|| GraphError::SlotNotFound(String::from(name)))
    }

    #[inline]
    fn get_slot_mut(&mut self, name: &str) -> Result<&mut EdgeSlot, GraphError> {
        self.slots
            .iter_mut()
            .find(|slot| slot.info.name() == name)
            .ok_or_else(|| GraphError::SlotNotFound(String::from(name)))
    }

    #[inline]
    fn check_set(&self) -> Result<(), GraphError> {
        for slot in &self.slots {
            if slot.value.is_none() {
                return Err(GraphError::SlotNotSet(String::from(slot.info.name())));
            }
        }

        Ok(())
    }

    #[inline]
    pub fn set<T: Any>(&mut self, name: &str, value: T) -> Result<(), GraphError> {
        let slot = self.get_slot_mut(name)?;

        if TypeId::of::<T>() != slot.info.ty_id() {
            return Err(GraphError::SetWrongType {
                expected: slot.info.ty_name(),
                found: type_name::<T>(),
            });
        }

        slot.value = Some(Box::new(value));

        Ok(())
    }

    #[inline]
    pub fn get<T: Any>(&self, name: &str) -> Result<&T, GraphError> {
        let slot = self.get_slot(name)?;

        let value = slot
            .value
            .as_ref()
            .ok_or_else(|| GraphError::SlotNotSet(String::from(name)))?;

        value
            .as_ref()
            .downcast_ref()
            .ok_or_else(|| GraphError::GetWrongType {
                found: slot.info.ty_name(),
                expected: type_name::<T>(),
            })
    }

    #[inline]
    pub fn get_mut<T: Any>(&mut self, name: &str) -> Result<&mut T, GraphError> {
        let slot = self.get_slot_mut(name)?;

        let value = slot
            .value
            .as_mut()
            .ok_or_else(|| GraphError::SlotNotSet(String::from(name)))?;

        value
            .as_mut()
            .downcast_mut()
            .ok_or_else(|| GraphError::GetWrongType {
                found: slot.info.ty_name(),
                expected: type_name::<T>(),
            })
    }
}

pub trait RenderNode {
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
    input: Option<NodeEdge>,
    output: Option<NodeEdge>,
    node: Box<dyn RenderNode>,
}

pub struct RenderGraph {
    edges: HashMap<String, Vec<String>>,
    slots: HashMap<SlotConnection, Vec<SlotConnection>>,
    nodes: HashMap<String, NodeContainer>,
}

impl RenderGraph {}
