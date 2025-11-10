use korp_engine::{
    Core,
    input::{Input, KeyCode},
    renderer::Canvas,
};
use korp_math::{Flint, Vec2};

use crate::{
    commands::{Command, Spawn},
    ecs::cosmos::Cosmos,
};

pub struct Korp {
    cosmos: Cosmos,
    key_bindings: KeyBindings,
    commands: Vec<Command>,
    actions: Vec<Action>,
    toggle: bool,
}

struct KeyBindings {
    up: KeyCode,
    down: KeyCode,
    left: KeyCode,
    right: KeyCode,
    toggle: KeyCode,
    triangle: KeyCode,
    rectangle: KeyCode,
}

enum Action {
    Toggle,
    Command(Command),
}

impl Korp {
    pub fn new() -> Self {
        Self {
            cosmos: Cosmos::new(),
            key_bindings: KeyBindings::new(),
            commands: Vec::new(),
            actions: Vec::new(),
            toggle: false,
        }
    }
}

impl Core for Korp {
    fn update(&mut self) {
        self.commands.clear();

        while let Some(action) = self.actions.pop() {
            match action {
                Action::Toggle => {
                    self.toggle = !self.toggle;
                }
                Action::Command(command) => {
                    self.commands.push(command);
                }
            }
        }

        self.cosmos.update(&self.commands);
    }

    fn input(&mut self, input: &Input) {
        if input.down(&self.key_bindings.up) {
            for (entity, _) in self.cosmos.components.motions.iter() {
                self.actions
                    .push(Action::Command(Command::Accelerate(*entity)));
            }
        }

        if input.down(&self.key_bindings.down) {
            for (entity, _) in self.cosmos.components.motions.iter() {
                self.actions
                    .push(Action::Command(Command::Decelerate(*entity)));
            }
        }

        if input.down(&self.key_bindings.left) {
            for (entity, _) in self.cosmos.components.motions.iter() {
                self.actions
                    .push(Action::Command(Command::TurnLeft(*entity)));
            }
        }

        if input.down(&self.key_bindings.right) {
            for (entity, _) in self.cosmos.components.motions.iter() {
                self.actions
                    .push(Action::Command(Command::TurnRight(*entity)));
            }
        }

        if input.is_pressed(&self.key_bindings.toggle) {
            self.actions.push(Action::Toggle);
        }

        if input.is_pressed(&self.key_bindings.triangle) {
            self.actions.push(Action::Command(Command::Spawn(
                Spawn::Triangle,
                Vec2::new(
                    Flint::from_i16(input.mouse.x as i16),
                    Flint::from_i16(input.mouse.y as i16),
                ),
            )));
        }

        if input.is_pressed(&self.key_bindings.rectangle) {
            self.actions.push(Action::Command(Command::Spawn(
                Spawn::Rectangle,
                Vec2::new(
                    Flint::from_i16(input.mouse.x as i16),
                    Flint::from_i16(input.mouse.y as i16),
                ),
            )));
        }
    }

    fn render(&mut self, canvas: &mut Canvas) {
        self.cosmos.render(canvas, self.toggle);
    }
}

impl KeyBindings {
    fn new() -> Self {
        Self {
            up: KeyCode::ArrowUp,
            down: KeyCode::ArrowDown,
            left: KeyCode::ArrowLeft,
            right: KeyCode::ArrowRight,
            toggle: KeyCode::F1,
            triangle: KeyCode::Digit1,
            rectangle: KeyCode::Digit2,
        }
    }
}
