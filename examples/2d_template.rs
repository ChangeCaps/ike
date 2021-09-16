use ike::prelude::*;

#[derive(Default)]
pub struct GameState {}

impl State for GameState {}

fn main() {
    let mut app = App::new();

    let mut main_pass = app.renderer.pass_mut::<MainPass>().unwrap();

    main_pass.push(SpriteNode2d::new());

    app.run(GameState::default());
}
