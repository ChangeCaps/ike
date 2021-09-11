use std::collections::BTreeMap;

use glam::{Mat3, Vec2};

use crate::{id::Id, prelude::Texture};

pub struct Sprite {
    pub transform: Mat3,
    pub depth: f32,
    pub width: f32,
    pub height: f32,
    pub min: Vec2,
    pub max: Vec2,
    pub texture_id: Id<Texture>,
    pub view: ike_wgpu::TextureView,
}

#[derive(Default)]
pub struct Sprites {
    pub(crate) batches: BTreeMap<Id<Texture>, Vec<Sprite>>,
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
