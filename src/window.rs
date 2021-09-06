use glam::UVec2;

pub struct Window {
    pub size: UVec2,
    pub fullscreen: bool,
    pub maximized: bool,
    pub cursor_visible: bool,
    pub cursor_grab: bool,
}

impl Default for Window {
    #[inline]
    fn default() -> Self {
        Self {
            size: UVec2::new(0, 0),
            fullscreen: false,
            maximized: false,
            cursor_visible: true,
            cursor_grab: false,
        }
    }
}

impl Window {
    #[inline]
    pub fn pre_update(&mut self, window: &winit::window::Window) {
        let size = window.inner_size();
        self.size = UVec2::new(size.width.max(1), size.height.max(1));
        self.maximized = window.is_maximized();
    }

    #[inline]
    pub fn post_update(&self, window: &winit::window::Window) {
        if self.fullscreen {
            window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
        } else {
            window.set_fullscreen(None);
        }

        window.set_maximized(self.maximized);
        window.set_cursor_visible(self.cursor_visible);
        window.set_cursor_grab(self.cursor_grab).unwrap();
    }
}
