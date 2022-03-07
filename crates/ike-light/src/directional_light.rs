use bytemuck::{Pod, Zeroable};
use ike_ecs::{Component, SparseStorage};
use ike_math::{Mat4, Vec3};
use ike_render::{Color, Orthographic};

#[derive(Clone)]
pub struct DirectionalLight {
    pub illuminance: f32,
    pub direction: Vec3,
    pub color: Color,
    pub projection: Orthographic,
}

impl Component for DirectionalLight {
    type Storage = SparseStorage;
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self {
            illuminance: 32_000.0,
            direction: -Vec3::Y,
            color: Color::WHITE,
            projection: Orthographic {
                left: -50.0,
                right: 50.0,
                bottom: -50.0,
                top: 50.0,
                near: -100.0,
                far: 100.0,
            },
        }
    }
}

impl DirectionalLight {
    const APERTURE: f32 = 4.0;
    const SHUTTER_SPEED: f32 = 1.0 / 250.0;
    const SENSITIVITY: f32 = 100.0;

    pub fn view_matrix(&self) -> Mat4 {
        let direction = self.direction.normalize();

        if direction.dot(Vec3::Y) < 0.01 {
            Mat4::look_at_rh(
                Vec3::ZERO,
                self.direction.normalize(),
                Vec3::new(0.0, 1.0, 0.2),
            )
        } else {
            Mat4::look_at_rh(Vec3::ZERO, self.direction.normalize(), Vec3::Y)
        }
    }

    pub fn as_raw(&self, _transform: Mat4) -> RawDirectionalLight {
        let ev100 = f32::log2(Self::APERTURE * Self::APERTURE / Self::SHUTTER_SPEED)
            - f32::log2(Self::SENSITIVITY / 100.0);
        let exposure = 1.0 / (f32::powf(2.0, ev100) * 1.2);
        let intensity = self.illuminance * exposure;

        let view_proj = self.view_matrix() * self.projection.matrix();

        RawDirectionalLight {
            view_proj: view_proj.to_cols_array_2d(),
            color: (self.color * intensity).into(),
            dir_to_light: (-self.direction.normalize()).into(),
            _padding: [0; 4],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct RawDirectionalLight {
    view_proj: [[f32; 4]; 4],
    color: [f32; 4],
    dir_to_light: [f32; 3],
    _padding: [u8; 4],
}
