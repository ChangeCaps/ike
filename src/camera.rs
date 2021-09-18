use glam::{Mat4, UVec2, Vec3};

use crate::{
    id::{HasId, Id},
    prelude::Transform3d,
};

#[derive(Clone, Debug)]
pub struct Camera {
    pub id: Id<Camera>,
    pub position: Vec3,
    pub view: Mat4,
    pub proj: Mat4,
}

impl HasId<Camera> for Camera {
    #[inline]
    fn id(&self) -> Id<Camera> {
        self.id
    }
}

impl Camera {
    #[inline]
    pub fn view_proj(&self) -> Mat4 {
        self.proj * self.view.inverse()
    }
}

#[derive(Clone, Debug)]
pub struct PerspectiveProjection {
    pub id: Id<Camera>,
    pub fov: f32,
    pub aspect: f32,
    pub near: f32,
}

impl HasId<Camera> for PerspectiveProjection {
    #[inline]
    fn id(&self) -> Id<Camera> {
        self.id
    }
}

impl Default for PerspectiveProjection {
    #[inline]
    fn default() -> Self {
        Self {
            id: Id::new(),
            fov: std::f32::consts::FRAC_PI_2,
            aspect: 1.0,
            near: 0.1,
        }
    }
}

impl PerspectiveProjection {
    #[inline]
    pub fn scale(&mut self, size: UVec2) {
        let width = size.x as f32;
        let height = size.y as f32;

        self.aspect = width / height;
    }

    #[inline]
    pub fn proj_matrix(&self) -> Mat4 {
        Mat4::perspective_infinite_rh(self.fov, self.aspect, self.near)
    }
}

#[derive(Clone, Debug)]
pub struct OrthographicProjection {
    pub id: Id<Camera>,
    pub left: f32,
    pub bottom: f32,
    pub right: f32,
    pub top: f32,
    pub size: f32,
    pub near: f32,
    pub far: f32,
}

impl HasId<Camera> for OrthographicProjection {
    #[inline]
    fn id(&self) -> Id<Camera> {
        self.id
    }
}

impl Default for OrthographicProjection {
    #[inline]
    fn default() -> Self {
        Self {
            id: Id::new(),
            left: 1.0,
            bottom: 1.0,
            right: 1.0,
            top: 1.0,
            size: 2.0,
            near: -500.0,
            far: 500.0,
        }
    }
}

impl OrthographicProjection {
    #[inline]
    pub fn scale(&mut self, size: UVec2) {
        let width = size.x as f32;
        let height = size.y as f32;

        let aspect = width / height;

        self.left = -self.size * aspect / 2.0;
        self.bottom = -self.size / 2.0;
        self.right = self.size * aspect / 2.0;
        self.top = self.size / 2.0;
    }

    #[inline]
    pub fn proj_matrix(&self) -> Mat4 {
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

#[derive(Clone, Debug, Default)]
pub struct PerspectiveCamera {
    pub projection: PerspectiveProjection,
    pub transform: Transform3d,
    pub global_transform: Option<Transform3d>,
}

impl HasId<Camera> for PerspectiveCamera {
    #[inline]
    fn id(&self) -> Id<Camera> {
        self.projection.id()
    }
}

impl PerspectiveCamera {
    #[inline]
    pub fn transform(&mut self, transform: &Transform3d) {
        self.global_transform = Some(transform * &self.transform);
    }

    #[inline]
    pub fn global_transform(&self) -> &Transform3d {
        self.global_transform.as_ref().unwrap_or(&self.transform)
    }

    #[inline]
    pub fn camera(&self) -> Camera {
        let transform = self.global_transform();

        Camera {
            id: self.id(),
            position: self.transform.translation,
            view: transform.matrix(),
            proj: self.projection.proj_matrix(),
        }
    }
}
