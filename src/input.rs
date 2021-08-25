use std::{collections::HashSet, hash::Hash};

use glam::Vec2;

#[derive(Clone)]
pub struct Input<T: Eq + Hash + Clone> {
    pressed: HashSet<T>,
    down: HashSet<T>,
    released: HashSet<T>,
}

impl<T: Eq + Hash + Clone> Input<T> {
    #[inline]
    pub fn press(&mut self, event: T) {
        self.pressed.insert(event.clone());
        self.down.insert(event);
    }

    #[inline]
    pub fn release(&mut self, event: T) {
        self.down.remove(&event);
        self.released.insert(event);
    }

    #[inline]
    pub fn update(&mut self) {
        self.pressed.clear();
        self.released.clear();
    }

    #[inline]
    pub fn pressed(&self, event: &T) -> bool {
        self.pressed.contains(event)
    }

    #[inline]
    pub fn down(&self, event: &T) -> bool {
        self.down.contains(event)
    }

    #[inline]
    pub fn released(&self, event: &T) -> bool {
        self.released.contains(event)
    }

    #[inline]
    pub fn iter_pressed(&self) -> impl Iterator<Item = &T> {
        self.pressed.iter()
    }

    #[inline]
    pub fn iter_down(&self) -> impl Iterator<Item = &T> {
        self.down.iter()
    }

    #[inline]
    pub fn iter_released(&self) -> impl Iterator<Item = &T> {
        self.released.iter()
    }
}

impl<T: Eq + Hash + Clone> Default for Input<T> {
    #[inline]
    fn default() -> Self {
        Self {
            pressed: Default::default(),
            down: Default::default(),
            released: Default::default(),
        }
    }
}

#[derive(Default)]
pub struct Mouse {
    pub prev_position: Vec2,
    pub position: Vec2,
}
