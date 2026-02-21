use std::net::{IpAddr, Ipv4Addr};

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

pub struct Menu {
    actions: Vec<Action>,
    counter: u8,
    state: State,
    keybindings: KeyBindings,
}

struct KeyBindings {
    host: KeyCode,
    connect: KeyCode,
}

enum State {
    Idle,
    Host,
    HostAwait,
    Hosted { id: usize },
    Connect,
    ConnectAwait,
    Connected { id: usize },
}

enum Action {
    Transition(State),
}

const TIMEOUT: u8 = 12;

impl Menu {
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
            counter: 0,
            state: State::Idle,
            keybindings: KeyBindings {
                host: KeyCode::KeyH,
                connect: KeyCode::KeyC,
            },
        }
    }

    pub fn update(&mut self, bus: &mut Bus) {
        self.action(bus);

        match self.state {
            State::Idle => return,
            State::Host => {
                bus.send(NetworkIntent::Host);

                self.actions.push(Action::Transition(State::HostAwait));
            }
            State::HostAwait => {
                self.counter += 1;

                if self.counter > TIMEOUT {
                    self.actions.push(Action::Transition(State::Idle));
                }
            }
            State::Hosted { id } => {
                bus.send(NexusIntent::Transition(NexusState::Lobby {
                    id,
                    host: true,
                }));
            }
            State::Connect => {
                bus.send(NetworkIntent::Connect(IpAddr::V4(Ipv4Addr::LOCALHOST)));

                self.actions.push(Action::Transition(State::ConnectAwait));
            }
            State::ConnectAwait => {
                self.counter += 1;

                if self.counter > TIMEOUT {
                    self.actions.push(Action::Transition(State::Idle));
                }
            }
            State::Connected { id } => {
                bus.send(NexusIntent::Transition(NexusState::Lobby {
                    id,
                    host: false,
                }));
            }
        }
    }

    pub fn input(&mut self, input: &Input) {
        match self.state {
            State::Idle => {
                if input.is_pressed(&self.keybindings.host) {
                    self.actions.push(Action::Transition(State::Host));
                }

                if input.is_pressed(&self.keybindings.connect) {
                    self.actions.push(Action::Transition(State::Connect));
                }
            }
            _ => return,
        }
    }

    pub fn render(&mut self, _renderer: &mut Renderer, _alpha: f32) {}

    pub fn event(&mut self, event: &Event) {
        match self.state {
            State::HostAwait => {
                if let Event::Network(Network::Event(NetworkEvent::Hosted { id })) = event {
                    self.actions
                        .push(Action::Transition(State::Hosted { id: *id }));
                }
            }
            State::ConnectAwait => {
                if let Event::Network(Network::Event(NetworkEvent::Connected { id })) = event {
                    self.actions
                        .push(Action::Transition(State::Connected { id: *id }));
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
