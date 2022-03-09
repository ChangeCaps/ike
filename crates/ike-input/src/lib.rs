mod input;
mod keyboard;
mod mouse;

pub use input::*;
pub use keyboard::*;
pub use mouse::*;

use ike_app::{App, CoreStage, Plugin};

#[derive(Default)]
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(self, app: &mut App) {
        app.init_resource::<Input<Key>>();
        app.init_resource::<Input<MouseButton>>();

        app.add_event::<KeyboardInput>();
        app.add_event::<MouseButtonInput>();
        app.add_system_to_stage(keyboard_input_system, CoreStage::PreUpdate);
        app.add_system_to_stage(mouse_button_input_system, CoreStage::PreUpdate);
    }
}
