#![deny(unsafe_op_in_unsafe_fn)]

mod assets;
mod build;
mod file_browser;
mod project;
mod scenes;
mod ui;

use std::path::{Path, PathBuf};

use build::{BuildCommand, BuildMode};
use clap::{crate_authors, crate_version, Parser};
use file_browser::FileBrowser;
use ike::{prelude::*, render::RenderSurface};
use ike_egui::{EguiPlugin, EguiTextures};
use libloading::library_filename;
use scenes::Scenes;
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
        .init_resource::<Scenes>()
        .init_resource::<FileBrowser>()
        .add_plugin(RenderPlugin)
        .add_plugin(EguiPlugin)
        .add_startup_system(setup.system())
        .add_system(ui::ui_system.system());
}

fn setup(
    mut scenes: ResMut<Scenes>,
    mut egui_textures: ResMut<EguiTextures>,
    mut render_surface: ResMut<RenderSurface>,
) {
    let opts = Opts::parse();

    render_surface.configure().present_mode = wgpu::PresentMode::Fifo;

    egui_textures.insert(MainTexture);

    std::env::set_current_dir(&opts.path).unwrap();

    let lib_name = library_filename("ike_example");
    let lib_name = lib_name.to_str().unwrap();

    let lib_path = Path::new("target").join("debug").join(&lib_name);

    let mut build_command = BuildCommand::new();

    build_command.cargo_arg("--lib");
    build_command.build_mode(BuildMode::Debug);
    build_command.cfg("editor");

    scenes.load(lib_path);
}
