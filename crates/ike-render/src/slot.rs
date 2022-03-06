use std::{
    any::{type_name, Any, TypeId},
    borrow::Cow,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SlotLabel {
    Label(Cow<'static, str>),
    Index(usize),
}

impl SlotLabel {
    pub fn new(label: impl Into<Cow<'static, str>>) -> Self {
        Self::Label(label.into())
    }
}

impl From<String> for SlotLabel {
    fn from(label: String) -> Self {
        Self::new(label)
    }
}

impl From<&'static str> for SlotLabel {
    fn from(label: &'static str) -> Self {
        Self::new(label)
    }
}

impl From<usize> for SlotLabel {
    fn from(index: usize) -> Self {
        Self::Index(index)
    }
}

impl From<&SlotLabel> for SlotLabel {
    fn from(label: &SlotLabel) -> Self {
        label.clone()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlotType {
    type_name: &'static str,
    type_id: TypeId,
}

impl SlotType {
    pub fn new<T: 'static>() -> Self {
        Self {
            type_name: type_name::<T>(),
            type_id: TypeId::of::<T>(),
        }
    }

    pub fn type_name(&self) -> &'static str {
        self.type_name
    }

    pub fn type_id(&self) -> &TypeId {
        &self.type_id
    }

    pub fn is<T: 'static>(&self) -> bool {
        self.type_id == TypeId::of::<T>()
    }
}

pub struct SlotValue {
    value: Box<dyn Any + Send + Sync>,
    slot_type: SlotType,
}

impl SlotValue {
    pub fn new<T: Any + Send + Sync>(value: T) -> Self {
        Self {
            value: Box::new(value),
            slot_type: SlotType::new::<T>(),
        }
    }

    pub fn slot_type(&self) -> &SlotType {
        &self.slot_type
    }

    pub fn downcast<T: Any>(&self) -> Option<&T> {
        self.value.as_ref().downcast_ref()
    }

    pub fn downcast_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.value.as_mut().downcast_mut()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlotInfo {
    label: SlotLabel,
    ty: SlotType,
}

impl SlotInfo {
    pub fn new<T: 'static>(label: &'static str) -> Self {
        Self {
            label: label.into(),
            ty: SlotType::new::<T>(),
        }
    }

    pub fn label(&self) -> &SlotLabel {
        &self.label
    }

    pub fn ty(&self) -> &SlotType {
        &self.ty
    }
}

pub struct SlotInfos {
    slots: Vec<SlotInfo>,
}

impl SlotInfos {
    pub fn new(slots: Vec<SlotInfo>) -> Self {
        Self { slots }
    }

    pub fn len(&self) -> usize {
        self.slots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    pub fn get(&self, label: impl Into<SlotLabel>) -> Option<&SlotInfo> {
        let index = self.get_slot_index(label)?;

        Some(&self.slots[index])
    }

    pub fn get_slot_index(&self, label: impl Into<SlotLabel>) -> Option<usize> {
        let label = label.into();

        match label {
            SlotLabel::Label(_) => self.slots.iter().position(|info| info.label == label),
            SlotLabel::Index(index) => {
                if self.slots.len() > index {
                    Some(index)
                } else {
                    None
                }
            }
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &SlotInfo> {
        self.slots.iter()
    }
}
