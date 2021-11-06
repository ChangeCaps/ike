use crossbeam::queue::SegQueue;
use glam::Vec3;
use ike_render::Color;

pub(crate) static DEBUG_LINES: SegQueue<DebugLine> = SegQueue::new();

pub struct DebugLine {
    pub from: Vec3,
    pub to: Vec3,
    pub width: f32,
    pub use_depth: bool,
    pub color: Color,
}

impl Default for DebugLine {
    #[inline]
    fn default() -> Self {
        Self {
            from: Vec3::ZERO,
            to: Vec3::ZERO,
            width: 0.001,
            use_depth: false,
            color: Color::WHITE,
        }
    }
}

impl DebugLine {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn from(mut self, from: Vec3) -> Self {
        self.from = from;
        self
    }

    #[inline]
    pub fn to(mut self, to: Vec3) -> Self {
        self.to = to;
        self
    }

    #[inline]
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    #[inline]
    pub fn use_depth(mut self) -> Self {
        self.use_depth = true;
        self
    }

    #[inline]
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    #[inline]
    pub fn draw(self) {
        DEBUG_LINES.push(self);
    }
}
