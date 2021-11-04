use glam::Vec3;
use ike_render::Color;

#[derive(Clone, Debug)]
pub struct PointLight {
    pub color: Color,
    pub intensity: f32,
    pub range: f32,
    pub radius: f32,
}

impl Default for PointLight {
    #[inline]
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            intensity: 5.0,
            range: 5.0,
            radius: 50.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct DirectionalLight {
    pub direction: Vec3,
    pub color: Color,
    pub illuminance: f32,
}

impl Default for DirectionalLight {
    #[inline]
    fn default() -> Self {
        Self {
            direction: -Vec3::Y,
            color: Color::WHITE,
            illuminance: 24000.0,
        }
    }
}
