use std::collections::HashMap;

use ike_assets::Handle;
use ike_render::Texture;

#[derive(Default)]
pub struct EguiTextures {
    egui: HashMap<u64, Handle<Texture>>,
    textures: HashMap<Handle<Texture>, u64>,
    next_id: u64,
}

impl EguiTextures {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn insert(&mut self, texture: Handle<Texture>) -> egui::TextureId {
        let id = self.next_id;
        self.next_id += 1;
        self.egui.insert(id, texture.clone());
        self.textures.insert(texture, id);

        egui::TextureId::User(id)
    }

    #[inline]
    pub fn get_texture(&self, id: &egui::TextureId) -> Option<&Handle<Texture>> {
        let id = if let egui::TextureId::User(id) = id {
            id
        } else {
            return None;
        };

        self.egui.get(id)
    }

    #[inline]
    pub fn get_egui(&self, texture: &Handle<Texture>) -> Option<egui::TextureId> {
        Some(egui::TextureId::User(*self.textures.get(texture)?))
    }
}
