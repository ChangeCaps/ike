use std::sync::Arc;

use ike_math::Vec2;
use winit::dpi::LogicalPosition;

pub type ExternalWindowError = winit::error::ExternalError;

pub type RawWindow = winit::window::Window;

pub struct Window {
    pub(crate) raw: Arc<RawWindow>,
    pub(crate) cursor_locked: bool,
    pub(crate) cursor_visible: bool,
    pub(crate) cursor_position: Vec2,
    pub(crate) cursor_delta: Vec2,
}

impl Window {
    pub fn new(window: RawWindow) -> Self {
        Self {
            raw: Arc::new(window),
            cursor_locked: false,
            cursor_visible: true,
            cursor_position: Vec2::ZERO,
            cursor_delta: Vec2::ZERO,
        }
    }

    pub fn width(&self) -> u32 {
        self.raw.inner_size().width
    }

    pub fn height(&self) -> u32 {
        self.raw.inner_size().height
    }

    pub fn size(&self) -> Vec2 {
        Vec2::new(self.width() as f32, self.height() as f32)
    }

    pub fn cursor_delta(&self) -> Vec2 {
        self.cursor_delta
    }

    pub fn cursor_position(&self) -> Vec2 {
        self.cursor_position
    }

    pub fn set_cursor_position(&mut self, position: Vec2) {
        self.cursor_position = position;

        self.raw()
            .set_cursor_position(LogicalPosition::new(position.x, position.y))
            .unwrap();
    }

    pub fn cursor_locked(&self) -> bool {
        self.cursor_locked
    }

    pub fn set_cursor_locked(&mut self, cursor_locked: bool) {
        self.cursor_locked = cursor_locked;

        self.raw.set_cursor_grab(self.cursor_locked).unwrap();
    }

    pub fn cursor_visible(&self) -> bool {
        self.cursor_visible
    }

    pub fn set_cursor_visible(&mut self, cursor_visible: bool) {
        self.cursor_visible = cursor_visible;

        self.raw.set_cursor_visible(self.cursor_visible);
    }

    pub fn raw(&self) -> &RawWindow {
        &self.raw
    }

    pub fn request_redraw(&self) {
        self.raw.request_redraw();
    }
}
