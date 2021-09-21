use crate::{
    id::{HasId, Id},
    prelude::{Color, Texture},
};

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
pub struct PbrMaterial {
    pub(crate) id: Id<PbrMaterial>,
    pub albedo_texture: Option<Texture>,
    pub metallic_roughness_texture: Option<Texture>,
    pub normal_map: Option<Texture>,
    pub albedo: Color,
    pub roughness: f32,
    pub metallic: f32,
    pub reflectance: f32,
    pub emission: Color,
    pub shadow_softness: f32,
    pub shadow_softness_falloff: f32,
    pub shadow_blocker_samples: u32,
    pub shadow_pcf_samples: u32,
    pub filter_mode: ike_wgpu::FilterMode,
}

impl HasId<PbrMaterial> for PbrMaterial {
    #[inline]
    fn id(&self) -> Id<PbrMaterial> {
        self.id
    }
}

impl Default for PbrMaterial {
    #[inline]
    fn default() -> Self {
        Self {
            id: Id::new(),
            albedo_texture: None,
            metallic_roughness_texture: None,
            normal_map: None,
            albedo: Color::WHITE,
            roughness: 0.089,
            metallic: 0.3,
            reflectance: 0.5,
            emission: Color::BLACK,
            shadow_softness: 1.0,
            shadow_softness_falloff: 4.0,
            #[cfg(debug_assertions)]
            shadow_blocker_samples: 8,
            #[cfg(not(debug_assertions))]
            shadow_blocker_samples: 16,
            #[cfg(debug_assertions)]
            shadow_pcf_samples: 16,
            #[cfg(not(debug_assertions))]
            shadow_pcf_samples: 32,
            filter_mode: ike_wgpu::FilterMode::Linear,
        }
    }
}

impl PbrMaterial {
    #[inline]
    pub fn metal() -> Self {
        Self {
            id: Id::new(),
            metallic: 0.99,
            reflectance: 0.9,
            ..Default::default()
        }
    }
}

impl PbrMaterial {
    #[inline]
    pub fn raw(&self) -> PbrMaterialRaw {
        PbrMaterialRaw {
            albedo: self.albedo.into(),
            emission: self.emission.into(),
            roughness: self.roughness,
            metallic: self.metallic,
            reflectance: self.reflectance,
            shadow_softness: self.shadow_softness,
            shadow_softness_falloff: self.shadow_softness_falloff,
            shadow_blocker_samples: self.shadow_blocker_samples,
            shadow_pcf_samples: self.shadow_pcf_samples,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PbrMaterialRaw {
    pub albedo: [f32; 4],
    pub emission: [f32; 4],
    pub roughness: f32,
    pub metallic: f32,
    pub reflectance: f32,
    pub shadow_softness: f32,
    pub shadow_softness_falloff: f32,
    pub shadow_blocker_samples: u32,
    pub shadow_pcf_samples: u32,
}
