use glam::Vec2;
use ike_ecs::prelude::{EventReader, ResMut};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u16),
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Mouse {
    pub position: Vec2,
    pub motion: Vec2,
    pub scroll: Vec2,
}

impl Mouse {
    pub fn system(
        mut position: EventReader<MousePosition>,
        mut motion: EventReader<MouseMotion>,
        mut scroll: EventReader<MouseScroll>,
        mut mouse: ResMut<Mouse>,
    ) {
        mouse.motion = Vec2::ZERO;
        mouse.scroll = Vec2::ZERO;

        if let Some(position) = position.iter().last() {
            mouse.position = position.position;
        }

        for motion in motion.iter() {
            mouse.motion += motion.delta;
        }

        for scroll in scroll.iter() {
            mouse.scroll += scroll.delta;
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MousePosition {
    pub position: Vec2,
}

#[derive(Clone, Copy, Debug)]
pub struct MouseMotion {
    pub delta: Vec2,
}

#[derive(Clone, Copy, Debug)]
pub struct MouseScroll {
    pub delta: Vec2,
}
