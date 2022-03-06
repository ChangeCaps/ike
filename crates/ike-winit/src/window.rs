use std::sync::Arc;

pub type RawWindow = winit::window::Window;

pub struct Window {
    raw: Arc<RawWindow>,
}

impl Window {
    pub fn new(window: RawWindow) -> Self {
        Self {
            raw: Arc::new(window),
        }
    }

    pub fn width(&self) -> u32 {
        self.raw.inner_size().width
    }

    pub fn height(&self) -> u32 {
        self.raw.inner_size().height
    }

    pub fn request_redraw(&self) {
        self.raw.request_redraw();
    }
}
