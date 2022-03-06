use ike_ecs::{Component, SparseStorage};
use ike_math::{Mat4, Vec3};

#[derive(Clone, Copy, Debug)]
pub struct RawCamera {
    pub view: Mat4,
    pub proj: Mat4,
    pub position: Vec3,
}

impl Default for RawCamera {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl RawCamera {
    pub const IDENTITY: Self = Self {
        view: Mat4::IDENTITY,
        proj: Mat4::IDENTITY,
        position: Vec3::ZERO,
    };

    pub fn view_proj(&self) -> Mat4 {
        self.proj * self.view.inverse()
    }
}

pub trait Projection {
    fn matrix(&self, aspect: f32) -> Mat4;
}

pub struct Perspective {
    pub fov: f32,
    pub near: f32,
}

impl Projection for Perspective {
    fn matrix(&self, aspect: f32) -> Mat4 {
        Mat4::perspective_infinite_rh(self.fov, aspect, self.near)
    }
}

impl Default for Perspective {
    fn default() -> Self {
        Self {
            fov: std::f32::consts::FRAC_PI_2,
            near: 0.1,
        }
    }
}

pub struct Orthographic {}

pub struct Camera {
    projection: Box<dyn Projection + Send + Sync>,
}

impl Default for Camera {
    fn default() -> Self {
        Self::new(Perspective::default())
    }
}

impl Camera {
    pub fn new(projection: impl Projection + Send + Sync + 'static) -> Self {
        Self {
            projection: Box::new(projection),
        }
    }

    pub fn proj_matrix(&self, aspect: f32) -> Mat4 {
        self.projection.matrix(aspect)
    }
}

impl Component for Camera {
    type Storage = SparseStorage;
}
