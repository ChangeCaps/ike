use glam::UVec2;

#[derive(Debug)]
pub struct Window {
    raw: winit::window::Window,
}

impl Window {
    #[inline]
    pub fn from_raw(raw: winit::window::Window) -> Self {
        Self { raw }
    }

    #[inline]
    pub fn get_raw(&self) -> &winit::window::Window {
        &self.raw
    }

    #[inline]
    pub fn size(&self) -> UVec2 {
        let size = self.raw.inner_size();
        UVec2::new(size.width, size.height).max(UVec2::ONE)
    }

    #[inline]
    pub fn aspect(&self) -> f32 {
        let size = self.size();
        size.x as f32 / size.y as f32
    }
}
