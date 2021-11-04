use ike_assets::Handle;
use ike_render::{Color, Texture};

pub struct PbrMaterial {
    pub albedo_texture: Option<Handle<Texture>>,
    pub metallic_roughness_texture: Option<Handle<Texture>>,
    pub normal_map: Option<Handle<Texture>>,
    pub albedo: Color,
    pub roughness: f32,
    pub metallic: f32,
    pub reflectance: f32,
    pub emission: Color,
    pub shadow_softness: f32,
    pub shadow_softness_falloff: f32,
    pub shadow_blocker_samples: u32,
    pub shadow_pcf_samples: u32,
}

impl Default for PbrMaterial {
    #[inline]
    fn default() -> Self {
        Self {
            albedo_texture: None,
            metallic_roughness_texture: None,
            normal_map: None,
            albedo: Color::WHITE,
            roughness: 0.8,
            metallic: 0.01,
            reflectance: 0.5,
            emission: Color::BLACK,
            shadow_softness: 1.0,
            shadow_softness_falloff: 4.0,
            shadow_blocker_samples: 12,
            shadow_pcf_samples: 24,
        }
    }
}
