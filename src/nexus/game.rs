use std::collections::HashMap;

use korp_engine::{
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
        events::{
            CosmosEvent, CosmosIntent, Event, GameEvent, IntentEvent, NetworkEvent, NetworkIntent,
        },
    },
    ecs::{
        commands::{Command, SpawnKind},
        cosmos::Cosmos,
        entities::Entity,
    },
};

pub struct Game {
    id: usize,
    pid: Option<Entity>,
    ids: Vec<usize>,
    seed: u64,
    cosmos: Cosmos,
    camera: Camera,
    camera_target: Morph<Vec2<f32>>,
    toggle: bool,
    keybindings: KeyBindings,
    state: State,
    actions: Vec<Action>,
    commands: Vec<Command>,
    commands_history: Vec<Vec<Vec<Command>>>,
    tick: usize,
    id_idx: HashMap<usize, usize>,
}

#[derive(Debug, Clone)]
pub enum State {
    Running,
    Paused,
    Stalling,
}

#[derive(Debug, Clone)]
pub enum Action {
    Transition(State),
    Toggle,
    Pause,
    Resume,
}

struct KeyBindings {
    up: KeyCode,
    down: KeyCode,
    left: KeyCode,
    right: KeyCode,
    toggle: KeyCode,
    triangle: KeyCode,
    rectangle: KeyCode,
    pause: KeyCode,
}

const TICK_DELAY: usize = 2;

impl Game {
    pub fn new(id: usize, ids: Vec<usize>, seed: u64) -> Self {
        let bounds = Rectangle {
            x: Flint::new(50, 0),
            y: Flint::new(40, 0),
            width: Flint::new(700, 0),
            height: Flint::new(400, 0),
        };
        let spawn = Vec2::new(
            bounds.x + bounds.width / Flint::from_i16(2),
            bounds.y + bounds.height / Flint::from_i16(2),
        );
        let mut cosmos = Cosmos::new(bounds);
        let mut id_idx = HashMap::new();
        let mut commands_history = Vec::with_capacity(1024);

        for tick in 0..TICK_DELAY {
            commands_history.push(Vec::with_capacity(ids.len()));

            for _ in 0..ids.len() {
                commands_history[tick].push(Vec::new());
            }
        }

        for (idx, id) in ids.iter().enumerate() {
            cosmos.event(
                &(CosmosIntent::Spawn {
                    id: Some(*id),
                    kind: SpawnKind::Triangle,
                    centroid: spawn,
                })
                .into(),
            );

            id_idx.insert(*id, idx);
        }

        Self {
            id,
            pid: None,
            ids,
            seed,
            cosmos,
            camera: Camera::new(800.0, 600.0),
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
                pause: KeyCode::KeyP,
            },
            state: State::Running,
            actions: Vec::new(),
            commands: Vec::new(),
            commands_history,
            tick: 0,
            id_idx,
        }
    }

    pub fn update(&mut self, bus: &mut Bus) {
        self.prepare();
        self.action(bus);
        self.schedule();

        let State::Running = self.state else {
            return;
        };

        self.cosmos.update(bus, &self.commands_history[self.tick]);
        self.tick += 1;
    }

    pub fn input(&mut self, input: &Input) {
        match self.state {
            State::Running => self.input_running(input),
            State::Paused => self.input_paused(input),
            State::Stalling => self.input_stalling(input),
        }
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

        if let Event::Network(IntentEvent::Event(event)) = event {
            self.event_network(event);
            return;
        }

        if let Event::Cosmos(IntentEvent::Event(event)) = event {
            self.event_cosmos(event);
            return;
        }
    }

    fn event_network(&mut self, event: &NetworkEvent) {
        match event {
            NetworkEvent::Disconnected { id } => {
                self.ids.retain(|x| x == id);
                // TODO: signal the cosmos?
            }
            NetworkEvent::Commands { id, tick, commands } => {
                self.commands(id, tick, commands);
            }
            NetworkEvent::Paused => {
                self.actions.push(Action::Transition(State::Paused));
            }
            NetworkEvent::Resumed => {
                self.actions.push(Action::Transition(State::Running));
            }
            _ => (),
        }
    }

    fn event_cosmos(&mut self, event: &CosmosEvent) {
        match event {
            CosmosEvent::Spawned {
                id: Some(id),
                entity,
            } if *id == self.id => {
                self.pid = Some(*entity);
                self.cosmos
                    .event(&(CosmosIntent::TrackDeath(*entity).into()));
                self.cosmos
                    .event(&(CosmosIntent::TrackMovement(*entity).into()));

                if let Some(body) = self.cosmos.components().logic.bodies.get(entity) {
                    self.camera_target.old = body.old.centroid.into();
                    self.camera_target.new = body.new.centroid.into();
                }
            }
            CosmosEvent::TrackedDeath(entity) if Some(*entity) == self.pid => {
                // when player is dead, set the new as the old to prevent wobbling
                self.pid = None;
                self.camera_target.old = self.camera_target.new;
            }
            CosmosEvent::TrackedMovement { entity, centroid } if Some(*entity) == self.pid => {
                self.camera_target.old = self.camera_target.new;
                self.camera_target.new = (*centroid).into();
            }
            _ => (),
        }
    }

    fn action(&mut self, bus: &mut Bus) {
        while let Some(action) = self.actions.pop() {
            bus.send(GameEvent::Action(action.clone()));

            match action {
                Action::Transition(state) => {
                    self.state = state;
                    bus.send(GameEvent::Transitioned(self.state.clone()));
                }
                Action::Toggle => {
                    self.toggle = !self.toggle;
                    bus.send(GameEvent::Toggled(self.toggle));
                }
                Action::Pause => {
                    bus.send(NetworkIntent::Pause);
                }
                Action::Resume => {
                    bus.send(NetworkIntent::Resume);
                }
            }
        }

        let State::Running = self.state else {
            return;
        };

        // always send the current commands for this tick
        bus.send(NetworkIntent::Commands {
            tick: self.tick + TICK_DELAY,
            commands: std::mem::take(&mut self.commands),
        });
    }

    fn commands(&mut self, id: &usize, tick: &usize, commands: &[Command]) {
        // ensure we can support the requested tick
        if self.commands_history.len() == *tick {
            self.commands_history
                .resize_with(self.commands_history.len() * 2, || {
                    let mut v = Vec::with_capacity(self.ids.len());
                    v.push(Vec::new());
                    v
                });
        }

        let tick_commands = &mut self.commands_history[*tick];
        let idx = self.id_idx[id];

        tick_commands[idx] = Vec::from(commands);
    }

    fn prepare(&mut self) {
        // ensure we have received all commands, otherwise stall
        if self.commands_history.len() < self.tick {
            let State::Stalling = self.state else {
                self.actions.push(Action::Transition(State::Stalling));
                return;
            };

            return;
        }

        let tick_commands = &self.commands_history[self.tick];

        if tick_commands.len() < self.ids.len() {
            let State::Stalling = self.state else {
                self.actions.push(Action::Transition(State::Stalling));
                return;
            };

            return;
        }

        let State::Stalling = self.state else {
            return;
        };

        self.actions.push(Action::Transition(State::Running));
    }

    fn schedule(&mut self) {}

    fn input_running(&mut self, input: &Input) {
        if input.is_pressed(&self.keybindings.pause) {
            self.actions.push(Action::Pause);
        }

        if input.is_pressed(&self.keybindings.toggle) {
            self.actions.push(Action::Toggle);
        }

        if input.is_pressed(&self.keybindings.triangle) {
            self.commands.push(Command::Spawn {
                id: None,
                kind: SpawnKind::Triangle,
                centroid: Vec2::new(
                    Flint::from_i16(input.mouse.x as i16),
                    Flint::from_i16(input.mouse.y as i16),
                ),
            });
        }

        if input.is_pressed(&self.keybindings.rectangle) {
            self.commands.push(Command::Spawn {
                id: None,
                kind: SpawnKind::Rectangle,
                centroid: Vec2::new(
                    Flint::from_i16(input.mouse.x as i16),
                    Flint::from_i16(input.mouse.y as i16),
                ),
            });
        }

        let Some(pid) = self.pid else {
            return;
        };

        if input.is_down(&self.keybindings.up) {
            self.commands.push(Command::Accelerate(pid));
        }

        if input.is_down(&self.keybindings.down) {
            self.commands.push(Command::Decelerate(pid));
        }

        if input.is_down(&self.keybindings.left) {
            self.commands.push(Command::TurnLeft(pid));
        }

        if input.is_down(&self.keybindings.right) {
            self.commands.push(Command::TurnRight(pid));
        }
    }

    fn input_stalling(&mut self, _input: &Input) {}

    fn input_paused(&mut self, input: &Input) {
        if input.is_pressed(&self.keybindings.pause) {
            self.actions.push(Action::Resume);
        }
    }
}
