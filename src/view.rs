use std::collections::HashMap;

use glam::Mat4;

use crate::id::Id;

pub struct View {
    pub id: Id,
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
    pub target_id: Option<Id>,
    pub views: HashMap<Id, View>,
}

impl Views {
    pub fn render_main_view(&mut self, id: Id, view_proj: Mat4) {
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
