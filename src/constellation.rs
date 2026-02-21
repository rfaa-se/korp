use crate::{bus::Bus, ecs::cosmos::Cosmos, network::Network, nexus::Nexus};

pub struct Constellation {
    pub nexus: Nexus,
    pub cosmos: Cosmos,
    pub network: Network,
}

impl Constellation {
    pub fn new() -> Self {
        Self {
            nexus: Nexus::new(),
            cosmos: Cosmos::new(),
            network: Network::new(),
        }
    }

    pub fn update(&mut self, bus: &mut Bus) {
        self.nexus.update(bus);
        self.network.update(bus);
        self.cosmos.update(bus);
    }
}
