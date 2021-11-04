#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Entity(pub(crate) u64);

impl Entity {
    #[inline]
    pub fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    #[inline]
    pub fn into_raw(self) -> u64 {
        self.0
    }
}
