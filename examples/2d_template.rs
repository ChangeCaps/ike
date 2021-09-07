use ike::{
    d2::render::{Render2d, Render2dCtx, SpriteNode2d},
    prelude::*,
};

#[derive(Default)]
pub struct GameState {}

impl State for GameState {}

impl Render2d for GameState {
    fn render(&mut self, ctx: &mut Render2dCtx) {}
}

fn main() {
    let mut app = App::new();

    let mut main_pass = app.renderer.pass_mut::<MainPass>().unwrap();

    main_pass.push(SpriteNode2d::new());

    app.run(GameState::default());
}
