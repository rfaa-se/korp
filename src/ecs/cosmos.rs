use korp_engine::{renderer::Renderer, shapes::Rectangle};
use korp_math::Flint;

use crate::{
    bus::{
        Bus,
        events::{CosmosIntent, Event, IntentEvent},
    },
    ecs::{
        commands::Command,
        components::Components,
        forge::Forge,
        systems::{Executor, Observer},
        tracker::Tracker,
    },
    quadtree::Quadtree,
};

pub struct Cosmos {
    bounds: Rectangle<Flint>,
    components: Components,
    forge: Forge,
    executor: Executor,
    observer: Observer,
    commands: Vec<Command>,
    tracker: Tracker,
    quadtree: Quadtree,
}

impl Cosmos {
    pub fn new(bounds: Rectangle<Flint>) -> Self {
        Self {
            bounds,
            components: Components::new(bounds),
            forge: Forge::new(),
            executor: Executor::new(),
            observer: Observer::new(),
            commands: Vec::new(),
            tracker: Tracker::new(),
            quadtree: Quadtree::new(bounds, 12, 8),
        }
    }

    pub fn update(&mut self, bus: &mut Bus, commands: &[Vec<Command>]) {
        self.execute_commands(bus, commands);

        self.executor.execute(
            &self.bounds,
            &mut self.components,
            &mut self.commands,
            &mut self.quadtree,
        );

        self.tracker.update(&self.components, bus);
    }

    pub fn render(&self, renderer: &mut Renderer, toggle: bool, alpha: f32) {
        self.observer
            .observe(&self.components, renderer, toggle, alpha);
    }

    pub fn event(&mut self, event: &Event) {
        let Event::Cosmos(IntentEvent::Intent(event)) = event else {
            return;
        };

        match event {
            CosmosIntent::Command(command) => {
                self.commands.push(command.clone());
            }
            CosmosIntent::Track(track) => {
                self.tracker.track(track);
            }
        }
    }

    pub fn components(&self) -> &Components {
        &self.components
    }

    fn execute_commands(&mut self, bus: &mut Bus, commands: &[Vec<Command>]) {
        for command in self.commands.drain(..) {
            command.execute(
                &mut self.components,
                &mut self.forge,
                &mut self.tracker,
                bus,
            );
        }

        for command in commands.iter().flatten() {
            command.execute(
                &mut self.components,
                &mut self.forge,
                &mut self.tracker,
                bus,
            );
        }
    }
}
