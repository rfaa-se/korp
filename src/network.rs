use std::net::IpAddr;

use crate::{
    bus::{
        Bus,
        events::{self, Event, NetworkEvent, NetworkIntent, NetworkResponse},
    },
    ecs::commands::Command,
};

pub struct Network {
    actions: Vec<Action>,
    id: usize,
}

pub enum Network2 {
    Void,
    Server,
    Client,
}

enum Action {
    Host,
    Connect(IpAddr),
    Disconnect,
    Commands { tick: usize, commands: Vec<Command> },
}

// TODO: this whole implementation is basically a stub
impl Network {
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
            id: 0,
        }
    }

    pub fn update(&mut self, bus: &mut Bus) {
        while let Some(action) = self.actions.pop() {
            match action {
                Action::Host => {
                    bus.send(NetworkResponse::Hosted);
                }
                Action::Connect(ip) => {
                    bus.send(NetworkResponse::Connected { ip, id: self.id });
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
            }
        }
    }

    pub fn event(&mut self, event: &Event) {
        let Event::Network(events::Network::Intent(event)) = event else {
            return;
        };

        match event {
            NetworkIntent::Host => self.actions.push(Action::Host),
            NetworkIntent::Connect(ip) => {
                self.actions.push(Action::Connect(*ip));
            }
            NetworkIntent::Commands { tick, commands } => {
                self.actions.push(Action::Commands {
                    tick: *tick,
                    commands: (*commands).clone(),
                });
            }
            NetworkIntent::Disconnect => {
                self.actions.push(Action::Disconnect);
            }
        }
    }
}
