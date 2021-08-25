use egui::*;
use glam::UVec2;

use crate::{id::Id, prelude::UiPanel, renderer::Renderer, ui_panel::UiPanelCtx};

pub struct MainViewPanel;

impl<S> UiPanel<S> for MainViewPanel {
    fn show(&mut self, ctx: &UiPanelCtx<S>) {
        CentralPanel::default().show(ctx.egui_ctx, |ui| {
            let size = ui.available_size();

            if let Some(target_id) = ctx.views.target_id {
                ui.image(target_id.into(), size);
            }
        });
    }
}
