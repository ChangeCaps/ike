#![deny(unsafe_op_in_unsafe_fn)]

mod build;
mod load_app;
mod project;
mod ui;

use std::path::{Path, PathBuf};

use build::{BuildCommand, BuildMode};
use clap::{crate_authors, crate_version, Parser};
use ike::prelude::*;
use ike_egui::{EguiPlugin, EguiTextures};
use libloading::library_filename;
use load_app::LoadedApp;
use ui::{Inspector, MainTexture};

#[derive(Parser)]
#[clap(version = crate_version!(), author = crate_authors!())]
struct Opts {
    /// Path to project.
    #[clap(default_value = ".")]
    path: PathBuf,
}

#[ike::main]
fn main(app: &mut AppBuilder) {
    app.set_runner(ike::winit::WinitRunner)
        .init_resource::<Inspector>()
        .add_plugin(RenderPlugin)
        .add_plugin(EguiPlugin)
        .add_startup_system(setup.system())
        .add_system(ui::ui_system.system());
}

fn setup(commands: Commands, mut egui_textures: ResMut<EguiTextures>) {
    let opts = Opts::parse();

    egui_textures.insert(MainTexture);

    std::env::set_current_dir(&opts.path).unwrap();

    let lib_name = library_filename("ike_example");
    let lib_name = lib_name.to_str().unwrap();

    let lib_path = Path::new("target").join("debug").join(&lib_name);

    let mut build_command = BuildCommand::new();

    build_command.cargo_arg("--lib");
    build_command.build_mode(BuildMode::Debug);
    build_command.cfg("editor");

    println!("{}", build_command);

    let output = build_command.command().output().unwrap();

    println!("{}", String::from_utf8_lossy(&output.stderr));

    let loaded_app = unsafe { LoadedApp::load(&lib_path).unwrap() };

    commands.insert_resource(loaded_app);
}
