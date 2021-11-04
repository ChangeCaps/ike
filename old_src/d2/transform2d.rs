use glam::{Mat3, Mat4, Quat, Vec2};

#[derive(Clone, Debug, PartialEq)]
pub struct Transform2d {
    pub translation: Vec2,
    pub angle: f32,
    pub scale: Vec2,
}

impl Transform2d {
    pub const IDENTITY: Self = Self {
        translation: Vec2::ZERO,
        angle: 0.0,
        scale: Vec2::ONE,
    };

    #[inline]
    pub fn from_xy(x: f32, y: f32) -> Self {
        Self {
            translation: Vec2::new(x, y),
            ..Self::IDENTITY
        }
    }

    #[inline]
    pub const fn from_translation(translation: Vec2) -> Self {
        Self {
            translation,
            ..Self::IDENTITY
        }
    }

    #[inline]
    pub const fn from_angle(angle: f32) -> Self {
        Self {
            angle,
            ..Self::IDENTITY
        }
    }

    #[inline]
    pub const fn from_scale(scale: Vec2) -> Self {
        Self {
            scale,
            ..Self::IDENTITY
        }
    }

    #[inline]
    pub fn matrix(&self) -> Mat3 {
        Mat3::from_scale_angle_translation(self.scale, self.angle, self.translation)
    }

    #[inline]
    pub fn matrix4x4(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(
            self.scale.extend(1.0),
            Quat::from_rotation_z(self.angle),
            self.translation.extend(0.0),
        )
    }
}

impl Default for Transform2d {
    #[inline]
    fn default() -> Self {
        Self::IDENTITY
    }
}
