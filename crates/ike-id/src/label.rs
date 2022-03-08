use std::{
    any::type_name,
    borrow::Cow,
    hash::{Hash, Hasher},
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RawLabel {
    type_name: Cow<'static, str>,
    id: u64,
}

impl RawLabel {
    pub fn from_hash<T: Hash>(hash: &T) -> Self {
        let mut state = ahash::AHasher::new_with_keys(420, 69);
        hash.hash(&mut state);

        Self {
            type_name: type_name::<T>().into(),
            id: state.finish(),
        }
    }

    pub fn type_name(&self) -> &Cow<'static, str> {
        &self.type_name
    }

    pub fn id(&self) -> u64 {
        self.id
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
