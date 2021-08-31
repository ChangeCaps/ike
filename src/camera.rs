use glam::{Mat4, UVec2};

use crate::id::Id;

/// Marker type for [`Id`]s.
pub struct Camera;

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
