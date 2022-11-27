use ike_math::Vec3;

#[derive(Clone, Copy, Debug)]
pub struct Gravity(pub Vec3);

impl Gravity {
    #[inline]
    pub const fn new(value: Vec3) -> Self {
        Self(value)
    }

    #[inline]
    pub const fn from_xyz(x: f32, y: f32, z: f32) -> Self {
        Self(Vec3::new(x, y, z))
    }
}

impl Default for Gravity {
    #[inline]
    fn default() -> Self {
        Self(Vec3::new(0.0, -9.81, 0.0))
    }
}
