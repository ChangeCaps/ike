use std::{
    any::type_name,
    borrow::Cow,
    hash::{Hash, Hasher},
};

use crate::uuid;
use ike_reflect::Reflect;

#[derive(Clone, Debug, Reflect, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[uuid("7b8cb912-fad3-410e-a483-008c70e3423f")]
pub enum RawLabelKind {
    Variant(Cow<'static, str>),
    Id(u64),
}

#[derive(Clone, Debug, Reflect, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[uuid("ba4659f2-b951-4899-847f-777fe745d679")]
pub struct RawLabel {
    type_name: Cow<'static, str>,
    kind: RawLabelKind,
}

impl RawLabel {
    pub fn variant<T>(variant: Cow<'static, str>) -> Self {
        Self {
            type_name: Cow::Borrowed(type_name::<T>()),
            kind: RawLabelKind::Variant(variant.into()),
        }
    }

    pub fn from_hash<T: Hash>(hash: T) -> Self {
        let mut state = ahash::AHasher::new_with_keys(420, 69);
        hash.hash(&mut state);

        Self {
            type_name: Cow::Borrowed(type_name::<T>()),
            kind: RawLabelKind::Id(state.finish()),
        }
    }

    pub fn type_name(&self) -> &Cow<'static, str> {
        &self.type_name
    }

    pub fn kind(&self) -> &RawLabelKind {
        &self.kind
    }
}

impl std::fmt::Display for RawLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            RawLabelKind::Id(id) => write!(f, "{}({})", self.type_name, id),
            RawLabelKind::Variant(ref variant) => write!(f, "{}::{}", self.type_name, variant),
        }
    }
}

#[macro_export]
macro_rules! define_label {
    ($ident:ident) => {
        pub trait $ident {
            fn raw_label(&self) -> $crate::RawLabel;
        }
    };
}
