use korp_engine::{renderer::Renderer, shapes::Rectangle};
use korp_math::Flint;

use crate::{
    bus::{
        Bus,
        events::{CosmosEvent, CosmosIntent, Event, IntentEvent},
    },
    ecs::{
        commands::Command,
        components::Components,
        forge::Forge,
        systems::{Executor, Observer, Processor},
        tracker::Tracker,
    },
    quadtree::Quadtree,
};

#[derive(Debug, Clone)]
pub enum Configure {
    Toggle(Toggle),
}

#[derive(Debug, Clone)]
pub enum Toggle {
    DrawFilled,
    DrawQuadtree,
    DrawHitbox,
}

pub struct Cosmos {
    bounds: Rectangle<Flint>,
    components: Components,
    forge: Forge,
    executor: Executor,
    observer: Observer,
    processor: Processor,
    commands: Vec<Command>,
    events: Vec<CosmosEvent>,
    tracker: Tracker,
    quadtree: Quadtree,
    configuration: Configuration,
}

pub struct Configuration {
    pub draw_filled: bool,
    pub draw_quadtree: bool,
    pub draw_hitbox: bool,
}

impl Cosmos {
    pub fn new(bounds: Rectangle<Flint>) -> Self {
        Self {
            bounds,
            components: Components::new(bounds),
            forge: Forge::new(),
            executor: Executor::new(),
            observer: Observer::new(),
            processor: Processor::new(),
            commands: Vec::new(),
            events: Vec::new(),
            tracker: Tracker::new(),
            quadtree: Quadtree::new(bounds, 12, 8),
            configuration: Configuration {
                draw_filled: false,
                draw_quadtree: false,
                draw_hitbox: false,
            },
        }
    }

    pub fn update(&mut self, bus: &mut Bus, commands: &[Vec<Command>]) {
        self.execute_commands(commands);

        self.executor.execute(
            self.bounds,
            &mut self.components,
            &mut self.commands,
            &mut self.quadtree,
            &mut self.events,
        );

        self.processor.process(
            &mut self.components,
            &mut self.tracker,
            &mut self.events,
            bus,
        );

        self.tracker.update(&self.components, bus);
    }

    pub fn render(&self, renderer: &mut Renderer, alpha: f32) {
        self.observer
            .observe(&self.components, &self.configuration, renderer, alpha);
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
            CosmosIntent::Configure(configure) => {
                self.configuration.configure(configure);
            }
        }
    }

    pub fn components(&self) -> &Components {
        &self.components
    }

    fn execute_commands(&mut self, commands: &[Vec<Command>]) {
        for command in self.commands.drain(..) {
            command.execute(&mut self.components, &mut self.forge, &mut self.events);
        }

        for command in commands.iter().flatten() {
            command.execute(&mut self.components, &mut self.forge, &mut self.events);
        }
    }
}

impl Configuration {
    fn configure(&mut self, configure: &Configure) {
        match configure {
            Configure::Toggle(toggle) => match toggle {
                Toggle::DrawFilled => self.draw_filled = !self.draw_filled,
                Toggle::DrawQuadtree => self.draw_quadtree = !self.draw_quadtree,
                Toggle::DrawHitbox => self.draw_hitbox = !self.draw_hitbox,
            },
        }
    }
}
