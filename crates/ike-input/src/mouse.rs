use ike_ecs::{EventReader, ResMut};

use crate::Input;

pub struct MouseButtonInput {
    pub button: MouseButton,
    pub pressed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    Other(u16),
}

pub fn mouse_button_input_system(
    mut event_reader: EventReader<MouseButtonInput>,
    mut mouse_button_input: ResMut<Input<MouseButton>>,
) {
    mouse_button_input.clear();

    for event in event_reader.iter() {
        if event.pressed {
            mouse_button_input.press(event.button);
        } else {
            mouse_button_input.release(event.button);
        }
    }
}
