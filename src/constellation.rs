use korp_engine::{input::Input, renderer::Renderer};

use crate::{
    bus::{Bus, events::Event},
    ecs::cosmos::Cosmos,
    korp::Kernel,
};

pub struct Constellation {
    pub kernel: Kernel,
    pub cosmos: Cosmos,
}

impl Constellation {
    pub fn new() -> Self {
        Self {
            kernel: Kernel::new(),
            cosmos: Cosmos::new(),
        }
    }

    pub fn update(&mut self, bus: &mut Bus) {
        self.kernel.update(bus);
        self.cosmos.update(bus);
    }

    pub fn event(&mut self, event: &Event) {
        self.kernel.event(event);
        self.cosmos.event(event);
    }

    pub fn input(&mut self, input: &Input) {
        self.kernel.input(input, &self.cosmos);
    }

    pub fn render(&mut self, renderer: &mut Renderer, alpha: f32) {
        self.kernel.render(&self.cosmos, renderer, alpha);
    }
}
