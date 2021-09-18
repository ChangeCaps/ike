use std::borrow::Cow;

use crate::prelude::{Color, Texture};

/// Bit flags for the pbr shader.
#[repr(transparent)]
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, bytemuck::Pod, bytemuck::Zeroable,
)]
pub struct PbrFlags(pub u32);

impl PbrFlags {
    pub const EMPTY: Self = Self(0b0000_0000_0000_0000);
    pub const NORMAL_MAP: Self = Self(0b0000_0000_0000_0001);
    pub const SKINNED: Self = Self(0b0000_0000_0000_0010);
}

impl std::ops::BitOr<PbrFlags> for PbrFlags {
    type Output = PbrFlags;

    #[inline]
    fn bitor(self, rhs: PbrFlags) -> Self::Output {
        PbrFlags(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign<PbrFlags> for PbrFlags {
    #[inline]
    fn bitor_assign(&mut self, rhs: PbrFlags) {
        self.0 |= rhs.0;
    }
}

#[derive(Clone, Debug)]
pub struct PbrMaterial<'a> {
    pub albedo_texture: Option<Cow<'a, Texture>>,
    pub metallic_roughness_texture: Option<Cow<'a, Texture>>,
    pub normal_map: Option<Cow<'a, Texture>>,
    pub albedo: Color,
    pub roughness: f32,
    pub metallic: f32,
    pub reflectance: f32,
    pub emission: Color,
    pub filter_mode: ike_wgpu::FilterMode,
}

impl Default for PbrMaterial<'_> {
    #[inline]
    fn default() -> Self {
        Self {
            albedo_texture: None,
            metallic_roughness_texture: None,
            normal_map: None,
            albedo: Color::WHITE,
            roughness: 0.089,
            metallic: 0.01,
            reflectance: 0.5,
            emission: Color::BLACK,
            filter_mode: ike_wgpu::FilterMode::Linear,
        }
    }
}
