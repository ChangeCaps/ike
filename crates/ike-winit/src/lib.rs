mod runner;
mod window;

pub use runner::*;
pub use window::*;

use ike_app::{App, Plugin};

#[derive(Default)]
pub struct WinitPlugin;

impl Plugin for WinitPlugin {
    fn build(self, app: &mut App) {
        let (runner, window, surface, device, queue) = WinitRunner::new();

        app.insert_resource(window);
        app.insert_resource(surface);
        app.insert_resource(device);
        app.insert_resource(queue);

        app.with_runner(runner);
    }
}
