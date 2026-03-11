use std::net::{IpAddr, Ipv4Addr};

use korp_engine::{
    input::{Input, KeyCode},
    renderer::Renderer,
};

use crate::{
    bus::{
        Bus,
        events::{Event, IntentEvent, MenuEvent, NetworkEvent, NetworkIntent, NexusIntent},
    },
    nexus,
};

pub struct Menu {
    actions: Vec<Action>,
    state: State,
    keybindings: KeyBindings,
}

#[derive(Debug, Clone)]
pub enum State {
    Idle,
    HostAwait { counter: u8 },
    ConnectAwait { counter: u8 },
    ExitAwait { counter: u8 },
}
#[derive(Debug, Clone)]
pub enum Action {
    Transition(State),
    Host,
    Hosted { id: usize },
    Connect,
    Connected { id: usize },
}

struct KeyBindings {
    host: KeyCode,
    connect: KeyCode,
}

const TIMEOUT: u8 = 12;

impl Menu {
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
            state: State::Idle,
            keybindings: KeyBindings {
                host: KeyCode::KeyH,
                connect: KeyCode::KeyC,
            },
        }
    }

    pub fn update(&mut self, bus: &mut Bus) {
        while let Some(action) = self.actions.pop() {
            self.state.handle(action, bus);
        }

        self.state.update(bus);
    }

    pub fn input(&mut self, input: &Input) {
        match self.state {
            State::Idle => {
                if input.is_pressed(&self.keybindings.host) {
                    self.actions.push(Action::Host);
                }

                if input.is_pressed(&self.keybindings.connect) {
                    self.actions.push(Action::Connect);
                }
            }
            _ => (),
        }
    }

    pub fn render(&mut self, _renderer: &mut Renderer, _alpha: f32) {}

    pub fn event(&mut self, event: &Event) {
        match (&self.state, event) {
            (
                State::HostAwait { .. },
                Event::Network(IntentEvent::Event(NetworkEvent::Hosted { id })),
            ) => {
                self.actions.push(Action::Hosted { id: *id });
            }
            (
                State::ConnectAwait { .. },
                Event::Network(IntentEvent::Event(NetworkEvent::Connected { id })),
            ) => {
                self.actions.push(Action::Connected { id: *id });
            }
            (_, _) => (),
        }
    }
}

impl State {
    fn handle(&mut self, action: Action, bus: &mut Bus) {
        bus.send(MenuEvent::Action(action.clone()));

        match (&self, action) {
            (State::Idle, Action::Host) => {
                bus.send(NetworkIntent::Host);
                self.handle(Action::Transition(State::HostAwait { counter: 0 }), bus)
            }
            (State::Idle, Action::Connect) => {
                bus.send(NetworkIntent::Connect(IpAddr::V4(Ipv4Addr::LOCALHOST)));
                self.handle(Action::Transition(State::ConnectAwait { counter: 0 }), bus)
            }
            (State::ConnectAwait { .. }, Action::Connected { id }) => {
                bus.send(NexusIntent::Transition(nexus::State::Lobby {
                    id,
                    host: false,
                }));

                self.handle(Action::Transition(State::ExitAwait { counter: 0 }), bus)
            }
            (State::HostAwait { .. }, Action::Hosted { id }) => {
                bus.send(NexusIntent::Transition(nexus::State::Lobby {
                    id,
                    host: true,
                }));

                self.handle(Action::Transition(State::ExitAwait { counter: 0 }), bus)
            }
            (_, Action::Transition(state)) => {
                *self = state;
                bus.send(MenuEvent::Transitioned(self.clone()));
            }
            (_, _) => (),
        }
    }

    fn update(&mut self, bus: &mut Bus) {
        match self {
            State::HostAwait { counter }
            | State::ConnectAwait { counter }
            | State::ExitAwait { counter } => {
                *counter += 1;

                if *counter > TIMEOUT {
                    self.handle(Action::Transition(State::Idle), bus);
                }
            }
            _ => (),
        }
    }
}
