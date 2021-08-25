use std::collections::HashMap;

use glam::{Mat3, Vec2};

use crate::id::Id;

pub struct Sprite {
    pub transform: Mat3,
    pub depth: f32,
    pub width: u32,
    pub height: u32,
    pub min: Vec2,
    pub max: Vec2,
    pub texture_id: Id,
    pub view: wgpu::TextureView,
}

#[derive(Default)]
pub struct Sprites {
    pub(crate) batches: HashMap<Id, Vec<Sprite>>,
}

impl Sprites {
    #[inline]
    pub fn draw(&mut self, sprite: Sprite) {
        self.batches
            .entry(sprite.texture_id)
            .or_default()
            .push(sprite);
    }
}
