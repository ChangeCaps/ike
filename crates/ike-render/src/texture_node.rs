use ike_ecs::World;

use crate::{
    RenderContext, RenderGraphContext, RenderGraphResult, RenderNode, SlotInfo, TextureDescriptor,
    TextureView,
};

pub struct TextureNode {
    desc: TextureDescriptor<'static>,
}

impl TextureNode {
    pub const TEXTURE: &'static str = "texture";

    pub fn new(desc: TextureDescriptor<'static>) -> Self {
        Self { desc }
    }
}

impl RenderNode for TextureNode {
    fn input() -> Vec<SlotInfo> {
        vec![SlotInfo::new::<TextureView>(Self::TEXTURE)]
    }

    fn output() -> Vec<SlotInfo> {
        vec![SlotInfo::new::<TextureView>(Self::TEXTURE)]
    }

    fn run(
        &mut self,
        graph_context: &mut RenderGraphContext<'_>,
        render_context: &mut RenderContext,
        _world: &World,
    ) -> RenderGraphResult<()> {
        if let Ok(texture) = graph_context.get_input::<TextureView>(Self::TEXTURE) {
            self.desc.size.width = texture.width();
            self.desc.size.height = texture.height();
        }

        if let Some(view) = graph_context.get_output_mut::<TextureView>(Self::TEXTURE)? {
            if view.width() != self.desc.size.width || view.height() != self.desc.size.height {
                let texture = render_context.device.create_texture(&self.desc);

                *view = texture.create_view(&Default::default());
            }
        } else {
            let texture = render_context.device.create_texture(&self.desc);

            graph_context.set_output(texture.create_view(&Default::default()), Self::TEXTURE)?;
        }

        Ok(())
    }
}
