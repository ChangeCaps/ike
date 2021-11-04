use ike_core::*;
use ike_input::*;
use ike_winit::{Key, MouseButton};

#[inline]
fn to_egui_key(key: Key) -> Option<egui::Key> {
    use egui::Key as K;

    Some(match key {
        Key::A => K::A,
        Key::B => K::B,
        Key::C => K::C,
        Key::D => K::D,
        Key::E => K::E,
        Key::F => K::F,
        Key::G => K::G,
        Key::H => K::H,
        Key::I => K::I,
        Key::J => K::J,
        Key::K => K::K,
        Key::L => K::L,
        Key::M => K::M,
        Key::N => K::N,
        Key::O => K::O,
        Key::P => K::P,
        Key::Q => K::Q,
        Key::R => K::R,
        Key::S => K::S,
        Key::T => K::T,
        Key::U => K::U,
        Key::V => K::V,
        Key::W => K::W,
        Key::X => K::X,
        Key::Y => K::Y,
        Key::Z => K::Z,
        Key::Key0 | Key::Numpad0 => K::Num0,
        Key::Key1 | Key::Numpad1 => K::Num1,
        Key::Key2 | Key::Numpad2 => K::Num2,
        Key::Key3 | Key::Numpad3 => K::Num3,
        Key::Key4 | Key::Numpad4 => K::Num4,
        Key::Key5 | Key::Numpad5 => K::Num5,
        Key::Key6 | Key::Numpad6 => K::Num6,
        Key::Key7 | Key::Numpad7 => K::Num7,
        Key::Key8 | Key::Numpad8 => K::Num8,
        Key::Key9 | Key::Numpad9 => K::Num9,
        _ => return None,
    })
}

#[inline]
fn to_egui_mouse_button(mouse_button: MouseButton) -> Option<egui::PointerButton> {
    use egui::PointerButton as P;

    Some(match mouse_button {
        MouseButton::Left => P::Primary,
        MouseButton::Right => P::Secondary,
        MouseButton::Middle => P::Middle,
        _ => return None,
    })
}

pub fn egui_input_system(
	mut ctx: ResMut<egui::CtxRef>,
    mut input: ResMut<egui::RawInput>,
    key_input: Res<Input<Key>>,
    mouse_input: Res<Input<MouseButton>>,
	text_input: Res<TextInput>,
    mouse: Res<Mouse>,
) {
    let modifiers = egui::Modifiers {
        alt: key_input.down(&Key::LAlt) | key_input.down(&Key::RAlt),
        ctrl: key_input.down(&Key::LControl) | key_input.down(&Key::RControl),
        shift: key_input.down(&Key::LShift) | key_input.down(&Key::RShift),
        mac_cmd: key_input.down(&Key::LWin),
        command: key_input.down(&Key::LWin),
    };

    input.modifiers = modifiers;

    let mouse_pos = egui::Pos2::new(mouse.position.x, mouse.position.y);
    input.events.push(egui::Event::PointerMoved(mouse_pos));
    input.scroll_delta = egui::Vec2::new(mouse.wheel_delta.x, mouse.wheel_delta.y);

    for key in key_input.iter_pressed() {
        if let Some(key) = to_egui_key(*key) {
            input.events.push(egui::Event::Key {
                key,
                pressed: true,
                modifiers,
            });
        }
    }

    for key in key_input.iter_released() {
        if let Some(key) = to_egui_key(*key) {
            input.events.push(egui::Event::Key {
                key,
                pressed: false,
                modifiers,
            });
        }
    }

    for button in mouse_input.iter_pressed() {
        if let Some(button) = to_egui_mouse_button(*button) {
            input.events.push(egui::Event::PointerButton {
                pos: mouse_pos,
                pressed: true,
                button,
                modifiers,
            });
        }
    }

    for button in mouse_input.iter_released() {
        if let Some(button) = to_egui_mouse_button(*button) {
            input.events.push(egui::Event::PointerButton {
                pos: mouse_pos,
                pressed: false,
                button,
                modifiers,
            });
        }
    }

	for c in text_input.0.iter().cloned() {
		if !c.is_control() {
			input.events.push(egui::Event::Text(String::from(c)));
		}
	}

	ctx.begin_frame(input.take());
}
