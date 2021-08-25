use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id(pub u64);

impl Default for Id {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Id {
    #[inline]
    pub fn new() -> Self {
        let inner = NEXT_ID.fetch_add(1, Ordering::SeqCst);

        Self(inner)
    }
}

impl Into<egui::TextureId> for Id {
    #[inline]
    fn into(self) -> egui::TextureId {
        egui::TextureId::User(self.0)
    }
}

pub trait HasId {
    fn id(&self) -> Id;
}
