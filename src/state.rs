use winit::event::{MouseButton, VirtualKeyCode};

use crate::{
    input::{Input, Mouse},
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
}

pub trait State {
    #[inline]
    fn start(&mut self, _ctx: &mut StartCtx) {}

    #[inline]
    fn update(&mut self, _ctx: &mut UpdateCtx) {}

    #[inline]
    fn render(&mut self, _views: &mut Views) {}

    #[inline]
    fn exit(&mut self) -> bool {
        true
    }
}
