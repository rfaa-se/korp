use korp_engine::{
    input::{Input, KeyCode},
    renderer::Renderer,
};

use crate::{
    bus::{
        Bus,
        events::{Event, Internal, Network, NetworkEvent, NetworkIntent, NexusIntent},
    },
    nexus::{self},
};

pub struct Lobby {
    id: usize,
    host: bool,
    counter: u8,
    state: State,
    actions: Vec<Action>,
    keybindings: KeyBindings,
}

#[derive(Debug, Clone)]
pub enum State {
    Idle,
    Launch,
    LaunchAwait,
    Launched,
    Leave,
    TransitionAwait,
}

#[derive(Debug, Clone)]
pub enum Action {
    Transition(State),
}

struct KeyBindings {
    start: KeyCode,
    exit: KeyCode,
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
        self.schedule();
    }

    pub fn input(&mut self, input: &Input) {
        match self.state {
            State::Idle => {
                if input.is_pressed(&self.keybindings.start) {
                    self.actions.push(Action::Transition(State::Launch));
                }

                if input.is_pressed(&self.keybindings.exit) {
                    self.actions.push(Action::Transition(State::Leave));
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

    fn action(&mut self, bus: &mut Bus) {
        while let Some(action) = self.actions.pop() {
            match action.clone() {
                Action::Transition(state) => {
                    match self.state {
                        State::Launch => {
                            bus.send(NetworkIntent::Launch);
                        }
                        State::Launched => {
                            bus.send(NexusIntent::Transition(nexus::State::Game {
                                id: self.id,
                                ids: vec![self.id],
                                seed: 0,
                            }));
                        }
                        State::Leave => {
                            bus.send(NexusIntent::Transition(nexus::State::Menu));
                        }
                        _ => (),
                    }

                    self.state = state;
                    self.counter = 0;
                }
            }

            bus.send(Internal::Lobby(action));
        }
    }

    fn schedule(&mut self) {
        match self.state {
            State::Idle => return,
            State::Launch => {
                self.actions.push(Action::Transition(State::LaunchAwait));
            }
            State::Launched | State::Leave => {
                self.actions
                    .push(Action::Transition(State::TransitionAwait));
            }
            State::LaunchAwait | State::TransitionAwait => {
                self.counter += 1;

                if self.counter > TIMEOUT {
                    self.actions.push(Action::Transition(State::Idle));
                }
            }
        }
    }
}
