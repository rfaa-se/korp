use crate::{
    bus::events::{CosmosEvent, Event, IntentEvent},
    network::Network,
    nexus::Nexus,
};

pub mod events;

pub struct Bus {
    events: Vec<Event>,
}

impl Bus {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn update(&mut self, nexus: &mut Nexus, network: &mut Network) {
        for event in self.events.drain(..) {
            nexus.event(&event);
            network.event(&event);

            // // let's not print these...
            match event {
                Event::Cosmos(IntentEvent::Event(CosmosEvent::TrackedMovement { .. })) => continue,
                _ => (),
            }

            // println!("{:?}", event);
        }
    }

    pub fn send<T: Into<Event>>(&mut self, event: T) {
        self.events.push(event.into());
    }
}
