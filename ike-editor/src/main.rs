#![deny(unsafe_op_in_unsafe_fn)]

mod load_app;
mod project;
mod ui;

use ike::prelude::*;
use ike_egui::{EguiPlugin, EguiTextures};
use load_app::LoadedApp;
use ui::{Inspector, MainTexture};

#[ike::main]
fn main(app: &mut AppBuilder) {
    app.set_runner(WinitRunner)
        .init_resource::<Inspector>()
        .add_plugin(RenderPlugin)
        .add_plugin(EguiPlugin)
        .add_startup_system(setup.system())
        .add_system(ui::ui_system.system());
}

fn setup(commands: Commands, mut egui_textures: ResMut<EguiTextures>) {
    egui_textures.insert(MainTexture);

    std::env::set_current_dir("../../ike-example/").unwrap();

    let loaded_app = unsafe { LoadedApp::load("target/debug/ike_example.dll").unwrap() };

    commands.insert_resource(loaded_app);
}
