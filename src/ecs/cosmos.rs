use korp_engine::{renderer::Renderer, shapes::Rectangle};
use korp_math::Flint;

use crate::{
    commands::Command,
    ecs::{components::Components, forge::Forge, systems::Executor},
};

pub struct Cosmos {
    pub components: Components,
    forge: Forge,
    executor: Executor,
    // observer: Observer,
    bounds: Rectangle<Flint>,
}

impl Cosmos {
    pub fn new() -> Self {
        Self {
            components: Components::new(),
            forge: Forge::new(),
            executor: Executor::new(),
            // observer: Observer::new(),
            bounds: Rectangle {
                x: Flint::new(50, 0),
                y: Flint::new(40, 0),
                width: Flint::new(700, 0),
                height: Flint::new(400, 0),
            },
        }
    }

    pub fn update(&mut self, commands: &[Command]) {
        self.executor.execute(
            &mut self.components,
            &mut self.forge,
            &self.bounds,
            commands,
        );
    }

    // pub fn render(&self, renderer: &mut Renderer, toggle: bool, alpha: f32) {
    //     self.observer
    //         .observe(&self.components, renderer, &self.bounds, toggle, alpha);
    // }
}
