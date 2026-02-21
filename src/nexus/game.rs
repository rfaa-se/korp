use korp_engine::{
    color::Color,
    input::{Input, KeyCode},
    misc::Morph,
    renderer::{Camera, Renderer},
    shapes::Rectangle,
};
use korp_math::{Vec2, lerp};

use crate::{
    bus::{Bus, events::Event},
    ecs::cosmos::Cosmos,
};

pub struct Game {
    id: usize,
    cosmos: Cosmos,
    camera: Camera,
    camera_target: Morph<Vec2<f32>>,
    toggle: bool,
    keybindings: KeyBindings,
    state: State,
    actions: Vec<Action>,
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

enum State {
    Running,
    Paused,
}

enum Action {}

impl Game {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            cosmos: Cosmos::new(),
            camera: Camera::new(1000.0, 1000.0),
            camera_target: Morph::one(Vec2::new(0.0, 0.0)),
            toggle: false,
            keybindings: KeyBindings {
                up: KeyCode::ArrowUp,
                down: KeyCode::ArrowDown,
                left: KeyCode::ArrowLeft,
                right: KeyCode::ArrowRight,
                toggle: KeyCode::F1,
                triangle: KeyCode::Digit1,
                rectangle: KeyCode::Digit2,
            },
            state: State::Running,
            actions: Vec::new(),
        }
    }

    pub fn update(&mut self, bus: &mut Bus) {
        let State::Running = self.state else {
            return;
        };
    }

    pub fn input(&mut self, input: &Input) {
        let State::Running = self.state else {
            return;
        };
    }

    pub fn render(&mut self, renderer: &mut Renderer, alpha: f32) {
        {
            self.camera.reposition(Vec2::new(
                lerp(self.camera_target.old.x, self.camera_target.new.x, alpha),
                lerp(self.camera_target.old.y, self.camera_target.new.y, alpha),
            ));

            // render cosmos using the camera
            let scope = renderer.begin(&self.camera);
            self.cosmos.render(scope.renderer, self.toggle, alpha);
        }

        // render ui
        renderer.draw_rectangle_lines(
            Rectangle::from(800.0, 120.0, Vec2::new(400.0, 540.0)),
            Vec2::new(1.0, 0.0),
            Vec2::new(400.0, 540.0),
            Color::GREEN,
        );
    }

    pub fn event(&mut self, event: &Event) {
        self.cosmos.event(event);
    }
}
