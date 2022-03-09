use bytemuck::{Pod, Zeroable};
use ike_ecs::Component;
use ike_math::{Mat4, Vec3};
use ike_render::{Color, Orthographic};

#[derive(Component, Clone)]
pub struct DirectionalLight {
    pub illuminance: f32,
    pub direction: Vec3,
    pub color: Color,
    pub projection: Orthographic,
    pub shadow_softness: f32,
    pub shadow_falloff: f32,
    pub blocker_samples: u32,
    pub pcf_samples: u32,
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self {
            illuminance: 32_000.0,
            direction: -Vec3::Y,
            color: Color::WHITE,
            projection: Orthographic {
                left: -25.0,
                right: 25.0,
                bottom: -25.0,
                top: 25.0,
                near: -25.0,
                far: 25.0,
            },
            shadow_softness: 2.0,
            shadow_falloff: 4.0,
            blocker_samples: 16,
            pcf_samples: 48,
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
            .inverse()
        } else {
            Mat4::look_at_rh(Vec3::ZERO, self.direction.normalize(), Vec3::Y).inverse()
        }
    }

    pub fn as_raw(&self, _transform: Mat4) -> RawDirectionalLight {
        let ev100 = f32::log2(Self::APERTURE * Self::APERTURE / Self::SHUTTER_SPEED)
            - f32::log2(Self::SENSITIVITY / 100.0);
        let exposure = 1.0 / (f32::powf(2.0, ev100) * 1.2);
        let intensity = self.illuminance * exposure;

        let view_proj = self.projection.matrix() * self.view_matrix().inverse();

        RawDirectionalLight {
            view_proj: view_proj.to_cols_array_2d(),
            color: (self.color * intensity).into(),
            dir_to_light: (-self.direction.normalize()).into(),
            _padding: [0; 4],
            size: [
                self.projection.right - self.projection.left,
                self.projection.top - self.projection.bottom,
            ],
            near: self.projection.near,
            far: self.projection.far,
            shadow_softness: self.shadow_softness,
            shadow_falloff: self.shadow_falloff,
            blocker_samples: self.blocker_samples,
            pcf_samples: self.pcf_samples,
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
    size: [f32; 2],
    near: f32,
    far: f32,
    shadow_softness: f32,
    shadow_falloff: f32,
    blocker_samples: u32,
    pcf_samples: u32,
}
