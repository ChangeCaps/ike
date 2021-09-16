use ike::prelude::*;

struct Game {
    texture: Texture,
    main_camera: OrthographicProjection,
}

impl Game {
    pub fn new() -> Self {
        Self {
            texture: Texture::load("assets/Tulip.png").unwrap(),
            main_camera: OrthographicProjection::default(),
        }
    }
}

impl State for Game {
    fn update(&mut self, ctx: &mut UpdateCtx) {
        ctx.views
            .render_main_view(self.main_camera.id, self.main_camera.proj_matrix());
    }
}

fn main() {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Error)
        .init()
        .unwrap();

    let app = App::new();

    app.run(Game::new())
}
