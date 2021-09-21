use std::collections::HashMap;

use glam::UVec2;

use crate::{
    camera::Camera,
    id::{HasId, Id},
};

pub struct View {
    pub camera: Camera,
    pub target: ike_wgpu::TextureView,
    pub width: u32,
    pub height: u32,
    pub format: ike_wgpu::TextureFormat,
}

impl View {
    #[inline]
    pub fn size(&self) -> UVec2 {
        UVec2::new(self.width, self.height)
    }
}

pub struct Views {
    pub target: Option<ike_wgpu::TextureView>,
    pub width: u32,
    pub height: u32,
    pub format: ike_wgpu::TextureFormat,
    pub target_id: Option<Id<Camera>>,
    pub views: HashMap<Id<Camera>, View>,
}

impl Views {
    pub fn render_main_view(&mut self, camera: Camera) {
        self.target_id = Some(camera.id());
        self.views.insert(
            camera.id(),
            View {
                camera,
                target: self.target.take().unwrap(),
                width: self.width,
                height: self.height,
                format: self.format,
            },
        );
    }
}
