use std::{collections::HashSet, hash::Hash};

pub struct Input<T> {
    pressed: HashSet<T>,
    held: HashSet<T>,
    released: HashSet<T>,
}

impl<T> Default for Input<T> {
    fn default() -> Self {
        Self {
            pressed: Default::default(),
            held: Default::default(),
            released: Default::default(),
        }
    }
}

impl<T: Clone + Eq + Hash> Input<T> {
    pub fn press(&mut self, event: T) {
        self.pressed.insert(event.clone());
        self.held.insert(event);
    }

    pub fn release(&mut self, event: T) {
        self.held.remove(&event);
        self.released.insert(event);
    }

    pub fn clear(&mut self) {
        self.pressed.clear();
        self.released.clear();
    }

    pub fn pressed(&self, event: &T) -> bool {
        self.pressed.contains(event)
    }

    pub fn held(&self, event: &T) -> bool {
        self.held.contains(event)
    }

    pub fn released(&self, event: &T) -> bool {
        self.released.contains(event)
    }
}
