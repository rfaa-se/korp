use korp_engine::{
    input::{Input, KeyCode},
    renderer::Renderer,
};

use crate::{
    bus::{
        Bus,
        events::{Event, IntentEvent, LobbyEvent, NetworkEvent, NetworkIntent, NexusIntent},
    },
    nexus,
};

pub struct Lobby {
    host: bool,
    data: Data,
    state: State,
    actions: Vec<Action>,
    keybindings: KeyBindings,
}

#[derive(Debug, Clone)]
pub enum State {
    Idle,
    LaunchAwait { counter: u8 },
    ExitAwait { counter: u8 },
}

#[derive(Debug, Clone)]
pub enum Action {
    Transition(State),
    Launch,
    Launched { seed: u64, delay: usize },
    Leave,
}

struct Data {
    id: usize,
    ids: Vec<usize>,
}

struct KeyBindings {
    start: KeyCode,
    exit: KeyCode,
}

const TIMEOUT: u8 = 12;

impl Lobby {
    pub fn new(id: usize, host: bool) -> Self {
        Self {
            host,
            data: Data { id, ids: vec![id] },
            state: State::Idle,
            actions: Vec::new(),
            keybindings: KeyBindings {
                start: KeyCode::KeyS,
                exit: KeyCode::KeyE,
            },
        }
    }

    pub fn update(&mut self, bus: &mut Bus) {
        while let Some(action) = self.actions.pop() {
            self.state.handle(action, bus, &self.data);
        }

        self.state.update(bus, &self.data);
    }

    pub fn input(&mut self, input: &Input) {
        match self.state {
            State::Idle { .. } => {
                if input.is_pressed(&self.keybindings.start) && self.host {
                    self.actions.push(Action::Launch);
                }

                if input.is_pressed(&self.keybindings.exit) {
                    self.actions.push(Action::Leave);
                }
            }
            _ => (),
        }
    }

    pub fn render(&mut self, _renderer: &mut Renderer, _alpha: f32) {}

    pub fn event(&mut self, event: &Event) {
        match (&self.state, event) {
            (
                State::LaunchAwait { .. },
                Event::Network(IntentEvent::Event(NetworkEvent::Launched { seed, delay })),
            ) => {
                self.actions.push(Action::Launched {
                    seed: *seed,
                    delay: *delay,
                });
            }
            _ => (),
        }
    }
}

impl State {
    fn handle(&mut self, action: Action, bus: &mut Bus, data: &Data) {
        bus.send(LobbyEvent::Action(action.clone()));

        match (&self, action) {
            (State::Idle, Action::Launch) => {
                bus.send(NetworkIntent::Launch);

                self.handle(
                    Action::Transition(State::LaunchAwait { counter: 0 }),
                    bus,
                    data,
                );
            }
            (State::Idle, Action::Leave) => {
                bus.send(NetworkIntent::Disconnect);
                bus.send(NexusIntent::Transition(nexus::State::Menu));

                self.handle(
                    Action::Transition(State::ExitAwait { counter: 0 }),
                    bus,
                    data,
                );
            }
            (State::LaunchAwait { .. }, Action::Launched { seed, delay }) => {
                bus.send(NexusIntent::Transition(nexus::State::Game {
                    id: data.id,
                    ids: data.ids.clone(),
                    seed,
                    delay,
                }));

                self.handle(
                    Action::Transition(State::ExitAwait { counter: 0 }),
                    bus,
                    data,
                );
            }
            (_, Action::Transition(state)) => {
                *self = state;
                bus.send(LobbyEvent::Transitioned(self.clone()));
            }
            (_, _) => (),
        }
    }

    fn update(&mut self, bus: &mut Bus, data: &Data) {
        match self {
            State::LaunchAwait { counter } | State::ExitAwait { counter } => {
                *counter += 1;

                if *counter > TIMEOUT {
                    self.handle(Action::Transition(State::Idle), bus, data);
                }
            }
            _ => (),
        }
    }
}
