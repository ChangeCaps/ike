use ike::prelude::*;
use ike_egui::EguiPlugin;

#[ike::main]
fn main(app: &mut AppBuilder) {
    app.set_runner(WinitRunner)
        .add_plugin(RenderPlugin)
        .add_plugin(EguiPlugin);
}
