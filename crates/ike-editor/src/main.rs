mod build;
mod egui_node;
mod logger;
mod panic;
mod project;

use build::BuildState;
use clap::{crate_authors, crate_version, Clap};
use egui_node::EguiNode;
use ike::{
    anyhow,
    app::AppTrait,
    editor_data::EditorData,
    egui::{Button, CtxRef, RawInput, TopBottomPanel},
    logger::Logger,
    prelude::*,
};
use libloading::{Library, Symbol};
use logger::LogReceiver;
use panic::{PanicHook, Panics};
use project::Project;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

pub struct AppState {
    pub raw_input: RawInput,
    pub egui_ctx: CtxRef,
    pub textures: HashMap<Id<Texture>, Texture>,
    pub panics: Panics,
    pub logger: LogReceiver,
    pub editor: EditorState,
}

impl AppState {
    pub fn top_bar_ui(&mut self) {
        let egui_ctx = &self.egui_ctx;
        let build_text = match self.editor.build_state {
            BuildState::Unloaded => "Build",
            BuildState::Building { .. } => "Building",
            BuildState::Loaded => "Rebuild",
        };
        let editor = &mut self.editor;

        TopBottomPanel::top("__ike_editor_top_bar__").show(egui_ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("File").clicked() {}

                if ui
                    .add(Button::new(build_text).enabled(!editor.building()))
                    .clicked()
                {
                    editor.build().unwrap();
                }
            });
        });
    }
}

#[derive(Default)]
pub struct EditorState {
    pub camera: OrthographicCamera,
    pub project: Project,
    pub path: PathBuf,
    pub build_state: BuildState,
    pub editor_data: EditorData,
    pub key_input: Input<Key>,
    pub mouse_input: Input<MouseButton>,
    pub window: Window,
    pub mouse: Mouse,
    pub views: Option<Views>,
    pub app: Option<Box<dyn AppTrait>>,
    pub lib: Option<Library>,
}

impl EditorState {
    pub fn load_project(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        Ok(Self {
            project: Project::load(path.as_ref())?,
            ..Default::default()
        })
    }

    pub fn exit_app(&mut self) {
        self.app.take();
    }

    pub fn unload(&mut self) {
        self.editor_data = EditorData::new();
        self.app.take();
        self.lib.take();
    }

    pub fn load(
        &mut self,
        panics: &Panics,
        logger: &LogReceiver,
        render_ctx: &RenderCtx,
        path: impl AsRef<Path>,
    ) -> anyhow::Result<()> {
        self.app = None;
        self.lib = None;

        let lib = unsafe { Library::new(path.as_ref())? };

        let hook = panics.hook();
        let logger = logger.logger();

        let export_app: Symbol<fn(PanicHook, Logger) -> ike::panic::Result<Box<dyn AppTrait>>> =
            unsafe { lib.get(b"export_app")? };
        let app = export_app(hook, logger);

        self.lib = Some(lib);

        if let Ok(mut app) = app {
            let mut start_ctx = StartCtx {
                render_ctx: &render_ctx,
                window: &mut self.window,
            };

            let _ = app.state().start(&mut start_ctx);

            self.app = Some(app);
        }

        Ok(())
    }
}

impl State for AppState {
    fn start(&mut self, ctx: &mut StartCtx) {
        self.editor.views = Some(Views {
            target: None,
            width: 0,
            height: 0,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            target_id: None,
            views: HashMap::new(),
        });

        ctx.window.title = String::from("Ike Editor");
    }

    fn update(&mut self, ctx: &mut UpdateCtx) {
        self.editor
            .update_build(&self.panics, &self.logger, ctx.render_ctx)
            .unwrap();

        for panic in self.panics.panics() {
            log::error!("app panicked:");

            if let Some(msg) = panic.message {
                log::error!("message: {}", msg);
            }

            if let Some(location) = panic.location {
                log::error!("location: {}", location);
            }
        }

        self.egui_input(ctx);

        self.egui_ctx.begin_frame(self.raw_input.take());

        self.top_bar_ui();

        if let Some(ref mut app) = self.editor.app {
            let (state, mut frame) = app.frame();

            let mut editor_ctx = EditorCtx {
                delta_time: ctx.delta_time,
                outer_window: &mut *ctx.window,
                inner_window: &mut self.editor.window,
                render_ctx: ctx.render_ctx,
                views: self.editor.views.as_mut().unwrap(),
                outer_mouse: &mut *ctx.mouse,
                inner_mouse: &mut self.editor.mouse,
                outer_key_input: &*ctx.key_input,
                inner_key_input: &mut self.editor.key_input,
                outer_mouse_input: &*ctx.mouse_input,
                inner_mouse_input: &mut self.editor.mouse_input,
                char_input: &*ctx.char_input,
                textures: &mut self.textures,
                egui_ctx: &self.egui_ctx,
                editor_data: &mut self.editor.editor_data,
                frame: &mut frame,
            };

            let _ = state.editor_update(
                &mut editor_ctx,
                Box::new(|state, editor_ctx| {
                    let mut update_ctx = UpdateCtx {
                        delta_time: editor_ctx.delta_time,
                        window: &mut *editor_ctx.inner_window,
                        key_input: &*editor_ctx.inner_key_input,
                        mouse_input: &*editor_ctx.inner_mouse_input,
                        mouse: &mut *editor_ctx.inner_mouse,
                        char_input: &*editor_ctx.char_input,
                        render_ctx: &*editor_ctx.render_ctx,
                        frame: &mut *editor_ctx.frame,
                        views: &mut *editor_ctx.views,
                    };

                    let _ = state.render(&mut update_ctx);
                }),
            );

            let _ = app.render_views(
                ctx.render_ctx,
                self.editor.views.as_ref().unwrap(),
                ctx.delta_time,
            );
        }
    }

    fn render(&mut self, ctx: &mut UpdateCtx) {
        ctx.views.render_main_view(self.editor.camera.camera());
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
        .with_module_level("naga", log::LevelFilter::Error)
        .init()
        .unwrap();

    let mut app = App::new();

    let mut main_pass = Pass::new(MainPass::default());
    main_pass.push(EguiNode::default());

    app.renderer.push_pass(main_pass);

    let mut state = EditorState::load_project(&opts.path)?;
    state.path = std::fs::canonicalize(opts.path)?;
    std::env::set_current_dir(&state.path)?;

    app.run(AppState {
        raw_input: RawInput::default(),
        egui_ctx: CtxRef::default(),
        textures: Default::default(),
        panics: Panics::new(),
        logger: LogReceiver::new(),
        editor: state,
    });
}
