use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
};

use egui::TextureId;
use ike_assets::{Assets, Handle};
use ike_core::WorldRef;
use ike_render::Texture;

pub trait EguiTextureHash {
    fn hash(&self) -> u64;
}

impl<T: Hash> EguiTextureHash for T {
    fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();

        Hash::hash(self, &mut hasher);

        hasher.finish()
    }
}

pub trait EguiTexture: EguiTextureHash + Send + Sync + 'static {
    fn get(&self, world: &WorldRef) -> Option<ike_wgpu::TextureView>;
}

impl EguiTexture for Handle<Texture> {
    fn get<'a>(&self, world: &WorldRef) -> Option<ike_wgpu::TextureView> {
        let textures = world.get_resource::<Assets<Texture>>()?;

        let texture = textures.get(self)?;

        Some(texture.texture().create_view(&Default::default()))
    }
}

#[derive(Default)]
pub struct EguiTextures {
    textures: HashMap<u64, Box<dyn EguiTexture>>,
}

impl EguiTextures {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn insert<T: EguiTexture>(&mut self, texture: T) -> TextureId {
        let hash = texture.hash();

        self.textures.insert(hash, Box::new(texture));

        TextureId::User(hash)
    }

    #[inline]
    pub fn get_id<T: EguiTexture>(&self, texture: &T) -> Option<TextureId> {
        Some(TextureId::User(texture.hash()))
    }

    #[inline]
    pub fn get_texture<'a>(
        &self,
        id: TextureId,
        world: &'a WorldRef,
    ) -> Option<ike_wgpu::TextureView> {
        let hash = match id {
            TextureId::User(id) => id,
            TextureId::Egui => return None,
        };

        self.textures.get(&hash)?.get(world)
    }
}
