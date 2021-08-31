use ike::{
    d2::{render::Render2d, transform2d::Transform2d},
    prelude::*,
};

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
    fn render(&mut self, views: &mut Views) {
        views.render_main_view(self.main_camera.id, self.main_camera.proj_matrix());
    }
}

impl Render2d for Game {
    fn render(&mut self, ctx: &mut ike::d2::render::Render2dCtx) {
        ctx.draw_texture(
            &mut self.texture,
            &Transform2d::from_scale(Vec2::splat(0.1)),
        );
    }
}

fn main() {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Error)
        .init()
        .unwrap();

    let app = App::new();

    //app.renderer.add_node(SpriteNode2d::new(Color::BLACK, 4));

    app.run(Game::new())
}
