use glam::{Mat3, Mat4, Quat, Vec3};
use ike_core::Entity;

#[derive(Clone, Debug, PartialEq)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {
    pub const IDENTITY: Self = Self {
        translation: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };

    #[inline]
    pub fn from_xyz(x: f32, y: f32, z: f32) -> Self {
        Self {
            translation: Vec3::new(x, y, z),
            ..Self::IDENTITY
        }
    }

    #[inline]
    pub const fn from_translation(translation: Vec3) -> Self {
        Self {
            translation,
            ..Self::IDENTITY
        }
    }

    #[inline]
    pub const fn from_rotation(rotation: Quat) -> Self {
        Self {
            rotation,
            ..Self::IDENTITY
        }
    }

    #[inline]
    pub const fn from_scale(scale: Vec3) -> Self {
        Self {
            scale,
            ..Self::IDENTITY
        }
    }

    #[inline]
    pub fn local_x(&self) -> Vec3 {
        self.rotation * Vec3::X
    }

    #[inline]
    pub fn local_y(&self) -> Vec3 {
        self.rotation * Vec3::Y
    }

    #[inline]
    pub fn local_z(&self) -> Vec3 {
        self.rotation * Vec3::Z
    }

    #[inline]
    pub fn mul_vec3(&self, mut vec3: Vec3) -> Vec3 {
        vec3 = self.rotation * vec3;
        vec3 *= self.scale;
        self.translation + vec3
    }

    #[inline]
    pub fn mul_transform(&self, other: &Self) -> Self {
        Self {
            translation: self.mul_vec3(other.translation),
            rotation: self.rotation * other.rotation,
            scale: self.scale * other.scale,
        }
    }

    #[inline]
    pub fn look_at(&mut self, target: Vec3, up: Vec3) {
        let forward = Vec3::normalize(self.translation - target);
        let right = up.cross(forward).normalize();
        let up = forward.cross(right);
        self.rotation = Quat::from_mat3(&Mat3::from_cols(right, up, forward));
    }

    #[inline]
    pub fn matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }
}

impl Default for Transform {
    #[inline]
    fn default() -> Self {
        Transform::IDENTITY
    }
}

impl std::ops::Mul<Transform> for Transform {
    type Output = Transform;

    #[inline]
    fn mul(self, rhs: Transform) -> Self::Output {
        self.mul_transform(&rhs)
    }
}

impl std::ops::Mul<&Transform> for Transform {
    type Output = Transform;

    #[inline]
    fn mul(self, rhs: &Transform) -> Self::Output {
        self.mul_transform(rhs)
    }
}

impl std::ops::Mul<Transform> for &Transform {
    type Output = Transform;

    #[inline]
    fn mul(self, rhs: Transform) -> Self::Output {
        self.mul_transform(&rhs)
    }
}

impl std::ops::Mul<&Transform> for &Transform {
    type Output = Transform;

    #[inline]
    fn mul(self, rhs: &Transform) -> Self::Output {
        self.mul_transform(rhs)
    }
}

impl std::ops::Mul<Vec3> for &Transform {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        self.mul_vec3(rhs)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GlobalTransform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl GlobalTransform {
    pub const IDENTITY: Self = Self {
        translation: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };

    #[inline]
    pub fn local_x(&self) -> Vec3 {
        self.rotation * Vec3::X
    }

    #[inline]
    pub fn local_y(&self) -> Vec3 {
        self.rotation * Vec3::Y
    }

    #[inline]
    pub fn local_z(&self) -> Vec3 {
        self.rotation * Vec3::Z
    }

    #[inline]
    pub fn mul_vec3(&self, mut vec3: Vec3) -> Vec3 {
        vec3 = self.rotation * vec3;
        vec3 *= self.scale;
        self.translation + vec3
    }

    #[inline]
    pub fn mul_transform(&self, other: &Transform) -> Self {
        Self {
            translation: self.mul_vec3(other.translation),
            rotation: self.rotation * other.rotation,
            scale: self.scale * other.scale,
        }
    }

    #[inline]
    pub fn matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }
}

impl From<Transform> for GlobalTransform {
    #[inline]
    fn from(transform: Transform) -> Self {
        Self {
            translation: transform.translation,
            rotation: transform.rotation,
            scale: transform.scale,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Parent(pub Entity);

#[derive(Clone, Debug)]
pub struct Children(pub Vec<Entity>);
