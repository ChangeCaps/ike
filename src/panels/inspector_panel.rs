use egui::{Response, SidePanel, Ui};

use crate::{
    id::{HasId, Id},
    prelude::UiPanel,
    ui_panel::UiPanelCtx,
};

pub trait Inspect: HasId {
    fn inspect(&mut self, ui: &mut Ui) -> Response;
}

pub struct InspectCtx<'a> {
    selected: &'a Id,
    ui: &'a mut Ui,
}

impl<'a> InspectCtx<'a> {
    pub fn inspect(&mut self, inspect: &mut impl Inspect) {
        if inspect.id() == *self.selected {
            inspect.inspect(self.ui);
        }
    }
}

pub trait Inspectable {
    fn inspect(&mut self, ctx: &mut InspectCtx);
}

#[derive(Default)]
pub struct InspectorPanel {
    pub selected: Option<Id>,
}

impl<S: Inspectable> UiPanel<S> for InspectorPanel {
    #[inline]
    fn show(&mut self, ctx: &mut UiPanelCtx<S>) {
        SidePanel::right("__ike_inspector_panel__")
            .resizable(true)
            .show(ctx.egui_ctx, |ui| {
                ui.heading("Inspector");
                ui.separator();

                if let Some(ref selected) = self.selected {
                    let mut inspect_ctx = InspectCtx { selected, ui };

                    ctx.state.inspect(&mut inspect_ctx);
                }
            });
    }
}
