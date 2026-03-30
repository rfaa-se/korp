use std::collections::HashMap;

use korp_engine::{
    color::Color,
    input::{Input, KeyCode},
    misc::Morph,
    renderer::{Camera, Renderer},
    shapes::Rectangle,
};
use korp_math::{Flint, Random, Vec2, lerp};

use crate::{
    bus::{
        Bus,
        events::{
            CosmosEvent, CosmosIntent, Event, GameEvent, IntentEvent, NetworkEvent, NetworkIntent,
        },
    },
    ecs::{
        commands::{Command, SpawnKind},
        cosmos::{Configure, Cosmos, Toggle},
        entities::Entity,
        tracker::Track,
    },
};

pub struct Game {
    random: Random,
    cosmos: Cosmos,
    camera: Camera,
    camera_target: Morph<Vec2<f32>>,
    keybindings: KeyBindings,
    state: State,
    actions: Vec<Action>,
    data: Data,
    alpha: f32,
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
    Toggle(Toggle),
    Pause,
    Paused,
    Resume,
    Resumed,
}

struct Data {
    id: usize,
    pid: Option<Entity>,
    ids: Vec<usize>,
    id_idx: HashMap<usize, usize>,
    tick: usize,
    commands: Vec<Command>,
    commands_history: Vec<Vec<Vec<Command>>>,
}

struct KeyBindings {
    up: KeyCode,
    down: KeyCode,
    left: KeyCode,
    right: KeyCode,
    toggle_draw_filled: KeyCode,
    toggle_draw_quadtree: KeyCode,
    toggle_draw_hitbox: KeyCode,
    triangle: KeyCode,
    rectangle: KeyCode,
    pause: KeyCode,
    shoot: KeyCode,
}

fn init_cosmos(
    cosmos: &mut Cosmos,
    commands_history: &mut Vec<Vec<Vec<Command>>>,
    id_idx: &mut HashMap<usize, usize>,
    ids: &[usize],
    delay: usize,
    spawn: Vec2<Flint>,
) {
    for tick in 0..delay {
        commands_history.push(Vec::with_capacity(ids.len()));

        for _ in 0..ids.len() {
            commands_history[tick].push(Vec::new());
        }
    }

    for (idx, id) in ids.iter().enumerate() {
        cosmos.event(
            &(CosmosIntent::Command(Command::Spawn {
                id: Some(*id),
                kind: SpawnKind::Triangle { centroid: spawn },
            }))
            .into(),
        );

        id_idx.insert(*id, idx);
    }
}

impl Game {
    pub fn new(id: usize, ids: Vec<usize>, seed: u64, delay: usize) -> Self {
        let bounds = Rectangle {
            x: Flint::new(50, 0),
            y: Flint::new(40, 0),
            width: Flint::new(1700, 0),
            height: Flint::new(1400, 0),
        };
        let spawn = Vec2::new(
            bounds.x + bounds.width / Flint::from_i16(2),
            bounds.y + bounds.height / Flint::from_i16(2),
        );
        let mut cosmos = Cosmos::new(bounds);
        let mut id_idx = HashMap::new();
        let mut commands_history = Vec::with_capacity(1024);

        init_cosmos(
            &mut cosmos,
            &mut commands_history,
            &mut id_idx,
            &ids,
            delay,
            spawn,
        );

        Self {
            data: Data {
                id,
                pid: None,
                ids,
                id_idx,
                tick: 0,
                commands: Vec::new(),
                commands_history,
            },
            random: Random::new(seed),
            cosmos,
            camera: Camera::new(800.0, 600.0),
            camera_target: Morph::one(spawn.into()),
            keybindings: KeyBindings {
                up: KeyCode::ArrowUp,
                down: KeyCode::ArrowDown,
                left: KeyCode::ArrowLeft,
                right: KeyCode::ArrowRight,
                toggle_draw_filled: KeyCode::F1,
                toggle_draw_quadtree: KeyCode::F2,
                toggle_draw_hitbox: KeyCode::F3,
                triangle: KeyCode::Digit1,
                rectangle: KeyCode::Digit2,
                pause: KeyCode::KeyP,
                shoot: KeyCode::Space,
            },
            state: State::Running,
            actions: Vec::new(),
            alpha: 0.0,
        }
    }

    pub fn update(&mut self, bus: &mut Bus) {
        while let Some(action) = self.actions.pop() {
            self.state.handle(action, bus, &mut self.data);
        }

        self.state
            .update(bus, &mut self.data, &mut self.cosmos, &mut self.random);
    }

    pub fn input(&mut self, input: &Input) {
        match self.state {
            State::Running => self.input_running(input),
            State::Paused => self.input_paused(input),
            State::Stalling => self.input_stalling(input),
        }
    }

    pub fn render(&mut self, renderer: &mut Renderer, alpha: f32) {
        let alpha = match self.state {
            State::Running => {
                self.alpha = alpha;
                alpha
            }
            State::Paused | State::Stalling => self.alpha,
        };

        {
            if matches!(self.state, State::Running) {
                self.camera.reposition(Vec2::new(
                    lerp(self.camera_target.old.x, self.camera_target.new.x, alpha),
                    lerp(self.camera_target.old.y, self.camera_target.new.y, alpha),
                ));
            }

            // render cosmos using the camera
            let scope = renderer.begin(&self.camera);
            self.cosmos.render(scope.renderer, alpha);
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
                self.data.ids.retain(|x| x != id);
                // TODO: signal the cosmos?
            }
            NetworkEvent::Commands { id, tick, commands } => {
                self.commands(id, tick, commands);
            }
            NetworkEvent::Paused => {
                self.actions.push(Action::Paused);
            }
            NetworkEvent::Resumed => {
                self.actions.push(Action::Resumed);
            }
            _ => (),
        }
    }

    fn event_cosmos(&mut self, event: &CosmosEvent) {
        match event {
            CosmosEvent::Spawned {
                id: Some(id),
                entity,
            } if *id == self.data.id => {
                self.data.pid = Some(*entity);
                self.cosmos
                    .event(&(CosmosIntent::Track(Track::Death(*entity)).into()));
                self.cosmos
                    .event(&(CosmosIntent::Track(Track::Movement(*entity)).into()));

                if let Some(body) = self.cosmos.components().logic.bodies.get(entity) {
                    self.camera_target.old = body.old.centroid.into();
                    self.camera_target.new = body.new.centroid.into();
                }
            }
            CosmosEvent::TrackedDeath(entity) if Some(*entity) == self.data.pid => {
                // when player is dead, set the new as the old to prevent wobbling
                self.data.pid = None;
                self.camera_target.old = self.camera_target.new;
            }
            CosmosEvent::TrackedMovement { entity, centroid } if Some(*entity) == self.data.pid => {
                self.camera_target.old = self.camera_target.new;
                self.camera_target.new = (*centroid).into();
            }
            _ => (),
        }
    }

    fn commands(&mut self, id: &usize, tick: &usize, commands: &[Command]) {
        // ensure we can support the requested tick
        if self.data.commands_history.len() == *tick {
            self.data
                .commands_history
                .resize_with(self.data.commands_history.len() * 2, || {
                    let mut v = Vec::with_capacity(self.data.ids.len());
                    v.push(Vec::new());
                    v
                });
        }

        let tick_commands = &mut self.data.commands_history[*tick];
        let idx = self.data.id_idx[id];

        tick_commands[idx] = Vec::from(commands);
    }

    fn input_running(&mut self, input: &Input) {
        if input.is_pressed(&self.keybindings.pause) {
            self.actions.push(Action::Pause);
        }

        if input.is_pressed(&self.keybindings.toggle_draw_filled) {
            self.actions.push(Action::Toggle(Toggle::DrawFilled));
        }

        if input.is_pressed(&self.keybindings.toggle_draw_quadtree) {
            self.actions.push(Action::Toggle(Toggle::DrawQuadtree));
        }

        if input.is_pressed(&self.keybindings.toggle_draw_hitbox) {
            self.actions.push(Action::Toggle(Toggle::DrawHitbox));
        }

        if input.is_pressed(&self.keybindings.triangle) {
            self.data.commands.push(Command::Spawn {
                id: None,
                kind: SpawnKind::Triangle {
                    centroid: Vec2::new(
                        Flint::from_i16(input.mouse.x as i16),
                        Flint::from_i16(input.mouse.y as i16),
                    ),
                },
            });
        }

        if input.is_pressed(&self.keybindings.rectangle) {
            self.data.commands.push(Command::Spawn {
                id: None,
                kind: SpawnKind::Rectangle {
                    centroid: Vec2::new(
                        Flint::from_i16(input.mouse.x as i16),
                        Flint::from_i16(input.mouse.y as i16),
                    ),
                },
            });
        }

        let Some(pid) = self.data.pid else {
            return;
        };

        if input.is_down(&self.keybindings.up) {
            self.data.commands.push(Command::Accelerate(pid));
        }

        if input.is_down(&self.keybindings.down) {
            self.data.commands.push(Command::Decelerate(pid));
        }

        if input.is_down(&self.keybindings.left) {
            self.data.commands.push(Command::TurnLeft(pid));
        }

        if input.is_down(&self.keybindings.right) {
            self.data.commands.push(Command::TurnRight(pid));
        }

        if input.is_down(&self.keybindings.shoot) {
            self.data.commands.push(Command::Shoot(pid));
        }
    }

    fn input_stalling(&mut self, _input: &Input) {}

    fn input_paused(&mut self, input: &Input) {
        if input.is_pressed(&self.keybindings.pause) {
            self.actions.push(Action::Resume);
        }
    }
}

impl State {
    fn handle(&mut self, action: Action, bus: &mut Bus, data: &mut Data) {
        bus.send(GameEvent::Action(action.clone()));

        match (&self, action) {
            (State::Running, Action::Pause) => {
                bus.send(NetworkIntent::Pause);
            }
            (State::Running, Action::Paused) => {
                self.handle(Action::Transition(State::Paused), bus, data);
            }
            (State::Paused, Action::Resume) => {
                bus.send(NetworkIntent::Resume);
            }
            (State::Paused, Action::Resumed) => {
                self.handle(Action::Transition(State::Running), bus, data);
            }
            (_, Action::Transition(state)) => {
                *self = state;
                bus.send(GameEvent::Transitioned(self.clone()));
            }
            (_, Action::Toggle(toggle)) => {
                bus.send(CosmosIntent::Configure(Configure::Toggle(toggle)));
            }
            (_, _) => (),
        }
    }

    fn update(&mut self, bus: &mut Bus, data: &mut Data, cosmos: &mut Cosmos, random: &mut Random) {
        self.prepare(bus, data);

        if !matches!(self, State::Running) {
            return;
        }

        // always send the current commands for this tick
        bus.send(NetworkIntent::Commands {
            tick: data.tick,
            commands: std::mem::take(&mut data.commands),
        });

        cosmos.update(bus, random, &data.commands_history[data.tick]);
        data.tick += 1;
    }

    fn prepare(&mut self, bus: &mut Bus, data: &mut Data) {
        // ensure we have received all commands, otherwise stall
        let has_history = data.commands_history.len() > data.tick;
        if !has_history {
            if !matches!(self, State::Stalling) {
                self.handle(Action::Transition(State::Stalling), bus, data);
            }

            return;
        }

        let has_commands = data.commands_history[data.tick].len() >= data.ids.len();
        if !has_commands {
            if !matches!(self, State::Stalling) {
                self.handle(Action::Transition(State::Stalling), bus, data);
            }

            return;
        }

        if matches!(self, State::Stalling) {
            self.handle(Action::Transition(State::Running), bus, data);
        }
    }
}
