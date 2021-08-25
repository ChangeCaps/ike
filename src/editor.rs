use egui::CtxRef;

use crate::{
    prelude::{App, UiPanel},
    renderer::RenderCtx,
    ui_panel::{UiPanelCtx, UiPanels},
    view::Views,
};

pub struct Editor<S: 'static> {
    pub panels: UiPanels<S>,
}

impl<S> Editor<S> {
    #[inline]
    pub fn insert_panel<T: UiPanel<S>>(&mut self, panel: T) {
        self.panels.insert(panel);
    }
}

impl<S> Default for Editor<S> {
    #[inline]
    fn default() -> Self {
        Self {
            panels: Default::default(),
        }
    }
}

impl<S> App<S> {
    pub fn show_editor(
        &mut self,
        views: &Views,
        egui_ctx: &CtxRef,
        render_ctx: &RenderCtx,
        state: &mut S,
    ) {
        let ctx = UiPanelCtx {
            egui_ctx,
            render_ctx,
            renderer: &mut self.renderer,
            views: &views,
            state,
        };

        self.editor.panels.show(&ctx);
    }
}
