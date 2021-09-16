use winit::event::{MouseButton, VirtualKeyCode};

use crate::{
    input::{Input, Mouse},
    renderer::{Drawable, RenderCtx, RenderFrame},
    view::Views,
    window::Window,
};

pub struct StartCtx<'a> {
    pub window: &'a mut Window,
}

pub struct UpdateCtx<'a> {
    pub delta_time: f32,
    pub window: &'a mut Window,
    pub key_input: &'a Input<VirtualKeyCode>,
    pub mouse_input: &'a Input<MouseButton>,
    pub mouse: &'a Mouse,
    pub char_input: &'a [char],
    pub render_ctx: &'a RenderCtx,
    pub frame: RenderFrame<'a>,
    pub views: &'a mut Views,
}

impl<'a> UpdateCtx<'a> {
    #[inline]
    pub fn draw<D: Drawable>(&mut self, drawable: &D) {
        self.frame.draw(self.render_ctx, drawable);
    }
}

pub trait State {
    #[inline]
    fn start(&mut self, _ctx: &mut StartCtx) {}

    #[inline]
    fn update(&mut self, _ctx: &mut UpdateCtx) {}

    #[inline]
    fn render(&mut self, _ctx: &mut UpdateCtx) {}

    #[inline]
    fn exit(&mut self) -> bool {
        true
    }
}
