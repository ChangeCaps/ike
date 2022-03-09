use ike_ecs::{Component, SparseStorage};
use ike_math::{Mat4, Vec3, Vec4};

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

    pub fn project_point(&self, point: Vec4) -> Vec4 {
        self.view_proj() * point
    }
}

pub trait Projection {
    fn matrix(&self, aspect: f32) -> Mat4;
}

#[derive(Clone)]
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

#[derive(Clone)]
pub struct Orthographic {
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
    pub near: f32,
    pub far: f32,
}

impl Orthographic {
    pub fn matrix(&self) -> Mat4 {
        Mat4::orthographic_rh(
            self.left,
            self.right,
            self.bottom,
            self.top,
            self.near,
            self.far,
        )
    }
}

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
