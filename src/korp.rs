use korp_engine::{
    Core,
    color::Color,
    input::{Input, KeyCode},
    misc::Morph,
    renderer::{Camera, Renderer},
    shapes::Rectangle,
};
use korp_math::{Flint, Vec2, lerp};

use crate::{
    bus::{
        Bus,
        events::{self, CosmosEvent, CosmosRequest, Event, KernelEvent},
    },
    commands::{Command, SpawnKind},
    constellation::Constellation,
    ecs::{cosmos::Cosmos, entities::Entity},
};

pub struct Korp {
    constellation: Constellation,
    bus: Bus,
}

pub struct Kernel {
    key_bindings: KeyBindings,
    commands: Vec<Command>,
    actions: Vec<Action>,
    toggle: bool,
    camera: Camera,
    camera_target: Morph<Vec2<f32>>,
    player_id: Entity,
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
    Init,
}

impl Kernel {
    pub fn new() -> Self {
        Self {
            key_bindings: KeyBindings::new(),
            commands: Vec::new(),
            actions: Vec::new(),
            toggle: false,
            camera: Camera::new(1000.0, 1000.0),
            camera_target: Morph::one(Vec2::new(0.0, 0.0)),
            // TODO: need a way to track local player entity
            player_id: Entity {
                index: 0,
                generation: 0,
            },
        }
    }

    pub fn update(&mut self, bus: &mut Bus) {
        while let Some(action) = self.actions.pop() {
            match action {
                Action::Toggle => {
                    self.toggle = !self.toggle;
                }
                Action::Command(command) => {
                    self.commands.push(command);
                }
                Action::Init => {
                    bus.send(CosmosRequest::TrackMovement(self.player_id));
                    bus.send(CosmosRequest::TrackDeath(self.player_id));
                }
            }
        }

        bus.send(CosmosRequest::Commands(std::mem::take(&mut self.commands)));
    }

    pub fn event(&mut self, event: &Event) {
        if let Event::Cosmos(events::Cosmos::Event(CosmosEvent::TrackedMovement {
            entity,
            centroid,
        })) = event
        {
            // make sure the camera tracks the player
            if *entity == self.player_id {
                self.camera_target.old = self.camera_target.new;
                self.camera_target.new = (*centroid).into();
            }
        }

        if let Event::Cosmos(events::Cosmos::Event(CosmosEvent::TrackedDeath(entity))) = event {
            // when player is dead, set the new value as the old to prevent wobbling
            if *entity == self.player_id {
                self.camera_target.old = self.camera_target.new;
            }
        }

        let Event::Kernel(events::Kernel::Event(event)) = event else {
            return;
        };

        match event {
            KernelEvent::Resized { width, height } => {
                self.camera.resize(*width, *height);
            }
            KernelEvent::Init => {
                self.actions.push(Action::Init);
            }
            _ => return,
        }
    }

    pub fn input(&mut self, input: &Input, cosmos: &Cosmos) {
        if input.down(&self.key_bindings.up) {
            for (entity, _) in cosmos.components.logic.motions.iter() {
                self.actions
                    .push(Action::Command(Command::Accelerate(*entity)));
            }
        }

        if input.down(&self.key_bindings.down) {
            for (entity, _) in cosmos.components.logic.motions.iter() {
                self.actions
                    .push(Action::Command(Command::Decelerate(*entity)));
            }
        }

        if input.down(&self.key_bindings.left) {
            for (entity, _) in cosmos.components.logic.motions.iter() {
                self.actions
                    .push(Action::Command(Command::TurnLeft(*entity)));
            }
        }

        if input.down(&self.key_bindings.right) {
            for (entity, _) in cosmos.components.logic.motions.iter() {
                self.actions
                    .push(Action::Command(Command::TurnRight(*entity)));
            }
        }

        if input.is_pressed(&self.key_bindings.toggle) {
            self.actions.push(Action::Toggle);
        }

        if input.is_pressed(&self.key_bindings.triangle) {
            self.actions.push(Action::Command(Command::Spawn {
                kind: SpawnKind::Triangle,
                centroid: Vec2::new(
                    Flint::from_i16(input.mouse.x as i16),
                    Flint::from_i16(input.mouse.y as i16),
                ),
            }));
        }

        if input.is_pressed(&self.key_bindings.rectangle) {
            self.actions.push(Action::Command(Command::Spawn {
                kind: SpawnKind::Rectangle,
                centroid: Vec2::new(
                    Flint::from_i16(input.mouse.x as i16),
                    Flint::from_i16(input.mouse.y as i16),
                ),
            }));
        }
    }

    pub fn render(&mut self, cosmos: &Cosmos, renderer: &mut Renderer, alpha: f32) {
        {
            self.camera.reposition(Vec2::new(
                lerp(self.camera_target.old.x, self.camera_target.new.x, alpha),
                lerp(self.camera_target.old.y, self.camera_target.new.y, alpha),
            ));

            // render cosmos using the camera
            let scope = renderer.begin(&self.camera);
            cosmos.render(scope.renderer, self.toggle, alpha);
        }

        // render ui
        renderer.draw_rectangle_lines(
            Rectangle::from(800.0, 120.0, Vec2::new(400.0, 540.0)),
            Vec2::new(1.0, 0.0),
            Vec2::new(400.0, 540.0),
            Color::GREEN,
        );
    }
}

impl Korp {
    pub fn new() -> Self {
        Self {
            constellation: Constellation::new(),
            bus: Bus::new(),
        }
    }
}

impl Core for Korp {
    fn update(&mut self) {
        self.bus.update(&mut self.constellation);
        self.constellation.update(&mut self.bus);
    }

    fn input(&mut self, input: &Input) {
        self.constellation.input(input);
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.bus.send(KernelEvent::Resized {
            width: width as f32,
            height: height as f32,
        });
    }

    fn render(&mut self, renderer: &mut Renderer, alpha: f32) {
        self.constellation.render(renderer, alpha);
    }

    fn init(&mut self) {
        self.bus.send(KernelEvent::Init);
    }

    fn exit(&mut self) {
        self.bus.send(KernelEvent::Exit);
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
