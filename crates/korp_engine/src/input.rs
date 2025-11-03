use std::collections::HashSet;

use korp_math::Vec2;

use crate::misc::Morph;
pub use winit::keyboard::KeyCode;

pub struct Input {
    pub(crate) keyboard: Morph<HashSet<KeyCode>>,
    pub(crate) keyboard_down: HashSet<KeyCode>,
    pub mouse: Vec2<f32>,
}

impl Input {
    pub(crate) fn new() -> Self {
        Self {
            keyboard: Morph::new(HashSet::new(), HashSet::new()),
            keyboard_down: HashSet::new(),
            mouse: Vec2::new(0.0, 0.0),
        }
    }

    pub(crate) fn update(&mut self) {
        self.keyboard.old.clear();
        self.keyboard.old.extend(&self.keyboard.new);
        self.keyboard_down.clear();
    }

    pub fn is_pressed(&self, key: &KeyCode) -> bool {
        self.keyboard.new.contains(key) && !self.keyboard.old.contains(key)
    }

    pub fn is_down(&self, key: &KeyCode) -> bool {
        self.keyboard.new.contains(key)
    }

    pub fn is_released(&self, key: &KeyCode) -> bool {
        !self.keyboard.new.contains(key)
    }

    pub fn was_down(&self, key: &KeyCode) -> bool {
        self.keyboard_down.contains(key)
    }

    pub fn down(&self, key: &KeyCode) -> bool {
        self.was_down(key) || self.is_down(key)
    }
}
