use std::net::IpAddr;

use crate::{
    bus::{
        Bus,
        events::{Event, IntentEvent, NetworkEvent, NetworkIntent},
    },
    ecs::commands::Command,
};

pub struct Network {
    actions: Vec<Action>,
    id: usize,
    delay: usize,
}

#[derive(Debug, Clone)]
pub enum Action {
    Host,
    Connect(IpAddr),
    Disconnect,
    Commands { tick: usize, commands: Vec<Command> },
    Launch,
    Pause,
    Resume,
}

// TODO: this whole implementation is basically a stub
impl Network {
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
            id: 0,
            delay: 2,
        }
    }

    pub fn update(&mut self, bus: &mut Bus) {
        self.action(bus);
    }

    pub fn event(&mut self, event: &Event) {
        let Event::Network(IntentEvent::Intent(event)) = event else {
            return;
        };

        match event {
            NetworkIntent::Host => self.actions.push(Action::Host),
            NetworkIntent::Connect(ip) => {
                self.actions.push(Action::Connect(*ip));
            }
            NetworkIntent::Commands { tick, commands } => {
                self.actions.push(Action::Commands {
                    tick: (*tick) + self.delay,
                    commands: (*commands).clone(),
                });
            }
            NetworkIntent::Disconnect => {
                self.actions.push(Action::Disconnect);
            }
            NetworkIntent::Launch => {
                self.actions.push(Action::Launch);
            }
            NetworkIntent::Pause => {
                self.actions.push(Action::Pause);
            }
            NetworkIntent::Resume => {
                self.actions.push(Action::Resume);
            }
        }
    }

    fn action(&mut self, bus: &mut Bus) {
        while let Some(action) = self.actions.pop() {
            bus.send(NetworkEvent::Action(action.clone()));

            match action {
                Action::Host => {
                    bus.send(NetworkEvent::Hosted { id: self.id });
                }
                Action::Connect(_ip) => {
                    bus.send(NetworkEvent::Connected { id: self.id });
                }
                Action::Disconnect => {
                    bus.send(NetworkEvent::Disconnected { id: self.id });
                }
                Action::Commands { tick, commands } => {
                    bus.send(NetworkEvent::Commands {
                        id: self.id,
                        tick,
                        commands,
                    });
                }
                Action::Launch => {
                    bus.send(NetworkEvent::Launched {
                        seed: 1337,
                        delay: self.delay,
                    });
                }
                Action::Pause => {
                    bus.send(NetworkEvent::Paused);
                }
                Action::Resume => {
                    bus.send(NetworkEvent::Resumed);
                }
            }
        }
    }
}
