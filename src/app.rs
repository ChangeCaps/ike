use egui::CtxRef;

use crate::{
    editor::Editor,
    prelude::UpdateCtx,
    renderer::{RenderCtx, RenderFrame, Renderer},
    state::State,
    view::Views,
};

pub struct App<S: 'static> {
    pub editor: Editor<S>,
    pub renderer: Renderer<S>,
}

impl<S> App<S> {
    #[inline]
    pub fn new() -> Self {
        Self {
            editor: Default::default(),
            renderer: Default::default(),
        }
    }
}

pub struct AppContainer<S: 'static> {
    pub app: App<S>,
    pub state: S,
}

pub trait AppTrait {
    fn show_editor(&mut self, views: &Views, egui_ctx: &CtxRef, render_ctx: &RenderCtx);

    fn update(&mut self, update_ctx: &mut UpdateCtx);

    fn render(&mut self, update_ctx: &mut UpdateCtx);

    fn frame(&mut self) -> RenderFrame<'_>;

    fn render_views(&mut self, ctx: &RenderCtx, views: &Views);
}

impl<S: State> AppTrait for AppContainer<S> {
    fn show_editor(&mut self, views: &Views, egui_ctx: &CtxRef, render_ctx: &RenderCtx) {
        self.app
            .show_editor(views, egui_ctx, render_ctx, &mut self.state);
    }

    fn update(&mut self, update_ctx: &mut UpdateCtx) {
        self.state.update(update_ctx);
    }

    fn render(&mut self, update_ctx: &mut UpdateCtx) {
        self.state.render(update_ctx);
    }

    fn frame(&mut self) -> RenderFrame<'_> {
        self.app.renderer.frame()
    }

    fn render_views(&mut self, render_ctx: &RenderCtx, views: &Views) {
        for view in views.views.values() {
            self.app
                .renderer
                .render_view(render_ctx, view, &mut self.state);
        }
    }
}
