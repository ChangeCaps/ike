use ike::prelude::*;
use ike_egui::*;

fn egui_system(ctx: Res<egui::CtxRef>) {
    egui::Window::new("Egui window").show(&ctx, |ui| {});
}

fn main() {
    App::new()
        .set_runner(WinitRunner)
        .add_plugin(RenderPlugin)
        .add_plugin(EguiPlugin)
        .add_system(egui_system.system())
        .run()
}
