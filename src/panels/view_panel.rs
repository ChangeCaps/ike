use egui::*;

use crate::{prelude::UiPanel, ui_panel::UiPanelCtx};

pub struct MainViewPanel;

impl<S> UiPanel<S> for MainViewPanel {
    #[inline]
    fn show(&mut self, ctx: &mut UiPanelCtx<S>) {
        CentralPanel::default().show(ctx.egui_ctx, |ui| {
            let size = ui.available_size();

            if let Some(target_id) = ctx.views.target_id {
                ui.image(target_id.into(), size);
            }
        });
    }
}
