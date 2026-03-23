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
    commands: Vec<Command>,
    tracker: Tracker,
    quadtree: Quadtree,
    config: Config,
}

pub struct Config {
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
            commands: Vec::new(),
            tracker: Tracker::new(),
            quadtree: Quadtree::new(bounds, 12, 8),
            config: Config {
                draw_filled: false,
                draw_quadtree: false,
                draw_hitbox: false,
            },
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

    pub fn render(&self, renderer: &mut Renderer, alpha: f32) {
        self.observer
            .observe(&self.components, renderer, &self.config, alpha);
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
            CosmosIntent::Configure(configure) => match configure {
                // TODO: impl Config { fn configure } ?
                Configure::Toggle(toggle) => match toggle {
                    Toggle::DrawFilled => self.config.draw_filled = !self.config.draw_filled,
                    Toggle::DrawQuadtree => self.config.draw_quadtree = !self.config.draw_quadtree,
                    Toggle::DrawHitbox => self.config.draw_hitbox = !self.config.draw_hitbox,
                },
            },
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
