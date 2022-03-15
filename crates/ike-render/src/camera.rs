use ike_ecs::component;
use ike_math::{Mat4, Vec3, Vec4};
use ike_reflect::Reflect;

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

#[derive(Clone, Reflect)]
pub struct Perspective {
    pub fov: f32,
    pub near: f32,
}

impl Default for Perspective {
    fn default() -> Self {
        Self {
            fov: std::f32::consts::FRAC_PI_2,
            near: 0.1,
        }
    }
}

impl Perspective {
    pub fn matrix(&self, aspect: f32) -> Mat4 {
        Mat4::perspective_infinite_rh(self.fov, aspect, self.near)
    }
}

#[derive(Clone, Reflect)]
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

#[derive(Reflect)]
pub enum Projection {
    Perspective(Perspective),
    Orthographic(Orthographic),
}

impl From<Perspective> for Projection {
    fn from(perspective: Perspective) -> Self {
        Self::Perspective(perspective)
    }
}

impl From<Orthographic> for Projection {
    fn from(orthographic: Orthographic) -> Self {
        Self::Orthographic(orthographic)
    }
}

impl Projection {
    pub fn matrix(&self, aspect: f32) -> Mat4 {
        match self {
            Self::Perspective(projection) => projection.matrix(aspect),
            Self::Orthographic(projection) => projection.matrix(),
        }
    }
}

#[component]
#[derive(Reflect)]
pub struct Camera {
    projection: Projection,
}

impl Default for Camera {
    fn default() -> Self {
        Self::new(Perspective::default())
    }
}

impl Camera {
    pub fn new(projection: impl Into<Projection>) -> Self {
        Self {
            projection: projection.into(),
        }
    }

    pub fn proj_matrix(&self, aspect: f32) -> Mat4 {
        self.projection.matrix(aspect)
    }
}
