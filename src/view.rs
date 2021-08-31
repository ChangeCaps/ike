use std::collections::HashMap;

use glam::Mat4;

use crate::{camera::Camera, id::Id};

pub struct View {
    pub id: Id<Camera>,
    pub view_proj: Mat4,
    pub target: wgpu::TextureView,
    pub width: u32,
    pub height: u32,
    pub format: wgpu::TextureFormat,
}

pub struct Views {
    pub target: Option<wgpu::TextureView>,
    pub width: u32,
    pub height: u32,
    pub format: wgpu::TextureFormat,
    pub target_id: Option<Id<Camera>>,
    pub views: HashMap<Id<Camera>, View>,
}

impl Views {
    pub fn render_main_view(&mut self, id: Id<Camera>, view_proj: Mat4) {
        self.target_id = Some(id);
        self.views.insert(
            id,
            View {
                id,
                view_proj,
                target: self.target.take().unwrap(),
                width: self.width,
                height: self.height,
                format: self.format,
            },
        );
    }
}
