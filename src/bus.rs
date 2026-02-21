use crate::{
    bus::events::{Cosmos, CosmosIntent, Event},
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
        for event in self.events.iter() {
            nexus.event(event);
            network.event(event);

            if let Event::Cosmos(Cosmos::Intent(CosmosIntent::PlayerCommands { .. })) = event {
                continue;
            }

            if let Event::Cosmos(Cosmos::Event(events::CosmosEvent::TrackedMovement {
                entity: _,
                centroid: _,
            })) = event
            {
                continue;
            }

            println!("{:?}", event);
        }

        self.events.clear();
    }

    pub fn send<T: Into<Event>>(&mut self, event: T) {
        self.events.push(event.into());
    }
}
