use korp_engine::{
    input::{Input, KeyCode},
    renderer::Renderer,
};

use crate::{
    bus::{
        Bus,
        events::{Event, Network, NetworkEvent, NetworkIntent, NexusIntent},
    },
    nexus::NexusState,
};

pub struct Lobby {
    id: usize,
    host: bool,
    counter: u8,
    state: State,
    actions: Vec<Action>,
    keybindings: KeyBindings,
}

struct KeyBindings {
    start: KeyCode,
    exit: KeyCode,
}

enum State {
    Idle,
    Launch,
    LaunchAwait,
    Launched,
    LaunchedAwait,
    Exit,
    ExitAwait,
}

enum Action {
    Transition(State),
}

const TIMEOUT: u8 = 12;

impl Lobby {
    pub fn new(id: usize, host: bool) -> Self {
        Self {
            id,
            host,
            counter: 0,
            state: State::Idle,
            actions: Vec::new(),
            keybindings: KeyBindings {
                start: KeyCode::KeyS,
                exit: KeyCode::KeyE,
            },
        }
    }

    pub fn update(&mut self, bus: &mut Bus) {
        self.action(bus);

        match self.state {
            State::Idle => return,
            State::Launch => {
                bus.send(NetworkIntent::Launch);

                self.actions.push(Action::Transition(State::LaunchAwait));
            }
            State::LaunchAwait => {
                self.counter += 1;

                if self.counter > TIMEOUT {
                    self.actions.push(Action::Transition(State::Idle));
                }
            }
            State::Launched => {
                bus.send(NexusIntent::Transition(NexusState::Game { id: self.id }));

                self.actions.push(Action::Transition(State::LaunchedAwait));
            }
            State::LaunchedAwait => {
                self.counter += 1;

                if self.counter > TIMEOUT {
                    // TOOD: something really wrong has happened
                    self.actions.push(Action::Transition(State::Idle));
                }
            }
            State::Exit => {
                bus.send(NexusIntent::Transition(NexusState::Menu));

                self.actions.push(Action::Transition(State::ExitAwait));
            }
            State::ExitAwait => {
                self.counter += 1;

                if self.counter > TIMEOUT {
                    self.actions.push(Action::Transition(State::Idle));
                }
            }
        }
    }

    pub fn input(&mut self, input: &Input) {
        match self.state {
            State::Idle => {
                if input.is_pressed(&self.keybindings.start) {
                    self.actions.push(Action::Transition(State::Launch));
                }

                if input.is_pressed(&self.keybindings.exit) {
                    self.actions.push(Action::Transition(State::Exit));
                }
            }
            _ => return,
        }
    }

    pub fn render(&mut self, _renderer: &mut Renderer, _alpha: f32) {}

    pub fn event(&mut self, event: &Event) {
        match self.state {
            State::LaunchAwait => {
                if let Event::Network(Network::Event(NetworkEvent::Launched)) = event {
                    self.actions.push(Action::Transition(State::Launched));
                }
            }
            _ => return,
        }
    }

    fn action(&mut self, _bus: &mut Bus) {
        while let Some(action) = self.actions.pop() {
            match action {
                Action::Transition(state) => {
                    self.state = state;
                    self.counter = 0;
                }
            }
        }
    }
}
