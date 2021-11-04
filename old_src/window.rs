use glam::UVec2;
use winit::dpi::PhysicalSize;

pub struct Window {
    pub size: UVec2,
    actual_size: UVec2,
    pub title: String,
    pub fullscreen: bool,
    pub maximized: bool,
    pub resizable: bool,
    pub cursor_visible: bool,
    pub cursor_grab: bool,
}

impl Default for Window {
    #[inline]
    fn default() -> Self {
        Self {
            size: UVec2::new(1, 1),
            actual_size: UVec2::new(1, 1),
            title: String::from("ike Application"),
            fullscreen: false,
            maximized: false,
            resizable: true,
            cursor_visible: true,
            cursor_grab: false,
        }
    }
}

impl Window {
    #[inline]
    pub fn pre_update(&mut self, window: &winit::window::Window) {
        let size = window.inner_size();
        // min size is [1, 1] to not cause wgpu to create textures with invalid sizes.
        self.size = UVec2::new(size.width.max(1), size.height.max(1));
        self.actual_size = self.size;

        self.maximized = window.is_maximized();
    }

    #[inline]
    pub fn post_update(&mut self, window: &winit::window::Window) {
        if self.fullscreen {
            window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
        } else {
            window.set_fullscreen(None);
        }

        if self.size != self.actual_size {
            let size = PhysicalSize::new(self.size.x.max(1), self.size.y.max(1));

            window.set_inner_size(size);
        }

        window.set_resizable(self.resizable);
        window.set_maximized(self.maximized);
        window.set_cursor_visible(self.cursor_visible);
        window.set_cursor_grab(self.cursor_grab).unwrap();
        window.set_title(&self.title);
    }
}
