use crate::App;

pub trait AppRunner {
    fn run(self: Box<Self>, app: App);
}

pub struct RunOnce;

impl AppRunner for RunOnce {
    fn run(self: Box<Self>, mut app: App) {
        app.update();
    }
}
