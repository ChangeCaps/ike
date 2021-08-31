mod egui_node;
mod project;

use clap::{crate_authors, crate_version, Clap};
use egui_node::{key_w2e, mouse_w2e, EguiNode};
use ike::{
    anyhow,
    app::AppTrait,
    egui::{Button, CtxRef, Event, Modifiers, Pos2, RawInput, TopBottomPanel},
    prelude::*,
    view::Views,
};
use libloading::{Library, Symbol};
use project::Project;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

#[derive(Default)]
pub struct EditorState {
    camera: OrthographicProjection,
    textures: HashMap<Id, wgpu::Texture>,
    raw_input: RawInput,
    egui_ctx: CtxRef,
    project: Project,
    app: Option<Box<dyn AppTrait>>,
    lib: Option<Library>,
}

impl EditorState {
    pub fn load_project(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        Ok(Self {
            project: Project::load(path.as_ref())?,
            ..Default::default()
        })
    }

    pub fn unload(&mut self) {
        self.app = None;
        self.lib = None;
    }

    pub fn load(&mut self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        self.app = None;
        self.lib = None;

        let lib = unsafe { Library::new(path.as_ref())? };

        let export_app: Symbol<fn() -> Box<dyn AppTrait>> = unsafe { lib.get(b"export_app")? };
        let app = export_app();

        self.lib = Some(lib);
        self.app = Some(app);

        Ok(())
    }

    pub fn top_bar_ui(&mut self) {
        TopBottomPanel::top("__ike_editor_top_bar__").show(&self.egui_ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("File").clicked() {}

                if ui.add(Button::new("Build")).clicked() {}
            });
        });
    }
}

impl State for EditorState {
    fn update(&mut self, ctx: &mut UpdateCtx) {
        let modifiers = Modifiers {
            alt: ctx.key_input.down(&Key::LAlt),
            ctrl: ctx.key_input.down(&Key::LControl),
            shift: ctx.key_input.down(&Key::LShift),
            mac_cmd: ctx.key_input.down(&Key::LWin),
            command: ctx.key_input.down(&Key::LWin),
        };

        for key in ctx.key_input.iter_pressed() {
            if let Some(key) = key_w2e(*key) {
                self.raw_input.events.push(Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                });
            }
        }

        for key in ctx.key_input.iter_released() {
            if let Some(key) = key_w2e(*key) {
                self.raw_input.events.push(Event::Key {
                    key,
                    pressed: false,
                    modifiers,
                });
            }
        }

        let pos = ike::egui::Pos2::new(ctx.mouse.position.x, ctx.mouse.position.y);

        for button in ctx.mouse_input.iter_pressed() {
            if let Some(button) = mouse_w2e(*button) {
                self.raw_input.events.push(Event::PointerButton {
                    pos,
                    button,
                    pressed: true,
                    modifiers,
                });
            }
        }

        for button in ctx.mouse_input.iter_released() {
            if let Some(button) = mouse_w2e(*button) {
                self.raw_input.events.push(Event::PointerButton {
                    pos,
                    button,
                    pressed: false,
                    modifiers,
                });
            }
        }

        for c in ctx.char_input {
            if !c.is_control() {
                self.raw_input.events.push(Event::Text(c.to_string()));
            }
        }

        self.raw_input.events.push(Event::PointerMoved(Pos2::new(
            ctx.mouse.position.x,
            ctx.mouse.position.y,
        )));
    }

    fn render(&mut self, views: &mut Views) {
        views.render_main_view(self.camera.id, self.camera.proj_matrix());
    }
}

#[derive(Clap)]
#[clap(author = crate_authors!(), version = crate_version!())]
struct Opts {
    path: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();

    simple_logger::SimpleLogger::new()
        .with_module_level("wgpu", log::LevelFilter::Error)
        .init()
        .unwrap();

    let mut app = App::new();

    let mut main_pass = app.renderer.pass_mut::<MainPass>().unwrap();
    main_pass.push(EguiNode::default());

    let state = EditorState::load_project(&opts.path)?;

    app.run(state);
}
