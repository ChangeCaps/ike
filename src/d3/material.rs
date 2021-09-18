use std::borrow::Cow;

use crate::prelude::{Color, Texture};

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

impl PbrMaterial<'_> {
    pub const NORMAL_MAP_FLAG: u32 = 1;

    #[inline]
    pub fn flags(&self, flags: &mut u32) {
        if self.normal_map.is_some() {
            *flags |= Self::NORMAL_MAP_FLAG;
        }
    }
}
