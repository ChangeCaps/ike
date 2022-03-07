use bytemuck::{Pod, Zeroable};
use ike_assets::Handle;
use ike_render::{Color, Image};

pub struct PbrMaterial {
    pub base_color: Color,
    pub base_color_texture: Option<Handle<Image>>,
    pub metallic: f32,
    pub roughness: f32,
    pub metallic_roughness_texture: Option<Handle<Image>>,
    pub reflectance: f32,
    pub emission: Color,
    pub emission_texture: Option<Handle<Image>>,
    pub normal_map: Option<Handle<Image>>,
}

impl Default for PbrMaterial {
    fn default() -> Self {
        Self {
            base_color: Color::WHITE,
            base_color_texture: None,
            metallic: 0.1,
            roughness: 0.981,
            metallic_roughness_texture: None,
            reflectance: 0.5,
            emission: Color::BLACK,
            emission_texture: None,
            normal_map: None,
        }
    }
}

impl PbrMaterial {
    pub fn as_raw(&self) -> RawPbrMaterial {
        RawPbrMaterial {
            base_color: self.base_color.into(),
            metallic: self.metallic,
            roughness: self.roughness,
            reflectance: self.reflectance,
            _padding: [0; 4],
            emission: self.emission.into(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct RawPbrMaterial {
    pub base_color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub reflectance: f32,
    pub _padding: [u8; 4],
    pub emission: [f32; 4],
}
