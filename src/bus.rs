use crate::{
    bus::events::{Cosmos, CosmosRequest, Event},
    constellation::Constellation,
};

pub mod events;

pub struct Bus {
    events: Vec<Event>,
}

impl Bus {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn update(&mut self, constellation: &mut Constellation) {
        for event in self.events.iter() {
            constellation.event(event);

            if let Event::Cosmos(Cosmos::Request(CosmosRequest::Commands(_))) = event {
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
