use korp_engine::{Core, CoreEvent, input::Input, renderer::Renderer};

use crate::{
    bus::{Bus, events::Event},
    network::Network,
    nexus::Nexus,
};

pub struct Korp {
    bus: Bus,
    nexus: Nexus,
    network: Network,
}

impl Korp {
    pub fn new() -> Self {
        Self {
            bus: Bus::new(),
            nexus: Nexus::new(),
            network: Network::new(),
        }
    }
}

impl Core for Korp {
    fn update(&mut self) {
        self.bus.update(&mut self.nexus, &mut self.network);
        self.network.update(&mut self.bus);
        self.nexus.update(&mut self.bus);
    }

    fn input(&mut self, input: &Input) {
        self.nexus.input(input);
    }

    fn render(&mut self, renderer: &mut Renderer, alpha: f32) {
        self.nexus.render(renderer, alpha);
    }

    fn event(&mut self, event: &CoreEvent) {
        self.bus.send(Event::Core(*event));
    }
}
