use std::{
    any::type_name,
    borrow::Cow,
    hash::{Hash, Hasher},
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RawLabelKind {
    Variant(Cow<'static, str>),
    Id(u64),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
