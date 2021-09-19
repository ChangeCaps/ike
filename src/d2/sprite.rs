use std::collections::BTreeMap;

use glam::{Mat4, Vec2};

use crate::{id::Id, prelude::Texture};

pub struct BatchedSprite {
    pub transform: Mat4,
    pub depth: f32,
    pub width: f32,
    pub height: f32,
    pub min: Vec2,
    pub max: Vec2,
    pub texture_id: Id<Texture>,
    pub filter_mode: ike_wgpu::FilterMode,
    pub view: ike_wgpu::TextureView,
}

#[derive(Default)]
pub struct Sprites {
    pub(crate) batches: BTreeMap<Id<Texture>, Vec<BatchedSprite>>,
}

impl Sprites {
    #[inline]
    pub fn draw(&mut self, sprite: BatchedSprite) {
        self.batches
            .entry(sprite.texture_id)
            .or_default()
            .push(sprite);
    }
}
