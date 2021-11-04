use std::{
    any::{type_name, Any, TypeId},
    borrow::Cow,
    collections::HashMap,
};

use crate::GraphError;

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
    value: Option<Box<dyn Any + Send + Sync>>,
    info: EdgeSlotInfo,
}

impl EdgeSlot {
    #[inline]
    pub fn new(info: EdgeSlotInfo) -> Self {
        Self { value: None, info }
    }
}

#[derive(Default)]
pub struct NodeInput<'a> {
    pub(crate) slots: HashMap<String, &'a EdgeSlot>,
}

impl<'a> NodeInput<'a> {
    #[inline]
    pub fn get<T: Any>(&self, name: &str) -> Result<&T, GraphError> {
        if let Some(slot) = self.slots.get(name) {
            slot.value
                .as_ref()
                .unwrap()
                .downcast_ref::<T>()
                .ok_or_else(|| GraphError::GetWrongType {
                    found: slot.info.ty_name(),
                    expected: type_name::<T>(),
                })
        } else {
            Err(GraphError::SlotNotFound(String::from(name)))
        }
    }
}

#[derive(Default)]
pub struct NodeEdge {
    slots: Vec<EdgeSlot>,
}

impl NodeEdge {
    #[inline]
    pub(crate) fn from_info(info: Vec<EdgeSlotInfo>) -> Self {
        Self {
            slots: info.into_iter().map(|info| EdgeSlot::new(info)).collect(),
        }
    }

    #[inline]
    pub(crate) fn get_slot(&self, name: &str) -> Result<&EdgeSlot, GraphError> {
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
    pub(crate) fn slots_set(&self) -> Result<(), GraphError> {
        for slot in &self.slots {
            if slot.value.is_none() {
                return Err(GraphError::SlotNotSet(String::from(slot.info.name())));
            }
        }

        Ok(())
    }

    #[inline]
    pub fn set<T: Any + Send + Sync>(&mut self, name: &str, value: T) -> Result<(), GraphError> {
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

    #[inline]
    pub fn remove<T: Any>(&mut self, name: &str) -> Result<T, GraphError> {
        let slot = self.get_slot_mut(name)?;

        let value = slot
            .value
            .take()
            .ok_or_else(|| GraphError::SlotNotSet(String::from(name)))?;

        value
            .downcast()
            .map(|t| *t)
            .map_err(|_| GraphError::GetWrongType {
                found: slot.info.ty_name(),
                expected: type_name::<T>(),
            })
    }
}
