use std::collections::HashMap;

use egui::CtxRef;
use winit::event::{MouseButton, VirtualKeyCode};

use crate::{
    editor_data::EditorData,
    id::Id,
    input::{Input, Mouse},
    texture::Texture,
    view::Views,
    window::Window,
};

pub struct StartCtx<'a> {
    pub window: &'a mut Window,
}

pub struct EditorCtx<'a> {
    pub delta_time: f32,
    pub outer_window: &'a mut Window,
    pub inner_window: &'a mut Window,
    pub outer_key_input: &'a Input<VirtualKeyCode>,
    pub inner_key_input: &'a mut Input<VirtualKeyCode>,
    pub outer_mouse_input: &'a Input<MouseButton>,
    pub inner_mouse_input: &'a mut Input<MouseButton>,
    pub outer_mouse: &'a mut Mouse,
    pub inner_mouse: &'a mut Mouse,
    pub char_input: &'a [char],
    pub views: &'a mut Views,
    pub textures: &'a mut HashMap<Id<Texture>, Texture>,
    pub egui_ctx: &'a CtxRef,
    pub editor_data: &'a mut EditorData,
}

pub struct UpdateCtx<'a> {
    pub delta_time: f32,
    pub window: &'a mut Window,
    pub key_input: &'a Input<VirtualKeyCode>,
    pub mouse_input: &'a Input<MouseButton>,
    pub mouse: &'a mut Mouse,
    pub char_input: &'a [char],
    pub views: &'a mut Views,
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
