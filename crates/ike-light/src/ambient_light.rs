use ike_render::Color;

#[derive(Clone)]
pub struct AmbientLight {
    pub color: Color,
    pub intensity: f32,
}

impl Default for AmbientLight {
    fn default() -> Self {
        Self {
            color: Color::rgb(0.5, 0.5, 0.7),
            intensity: 0.0005,
        }
    }
}
