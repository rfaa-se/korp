use korp_engine::{renderer::Renderer, shapes::Rectangle};
use korp_math::Flint;

use crate::{
    bus::{
        Bus,
        events::{self, CosmosEvent, CosmosIntent, Event},
    },
    ecs::{
        commands::Command,
        components::Components,
        entities::Entity,
        forge::Forge,
        systems::{Executor, Observer},
    },
};

pub struct Cosmos {
    components: Components,
    forge: Forge,
    executor: Executor,
    observer: Observer,
    bounds: Rectangle<Flint>,
    commands: Vec<Command>,
    tracked_death: Vec<Entity>,
    tracked_movement: Vec<Entity>,
    dead: Vec<Entity>,
}

impl Cosmos {
    pub fn new() -> Self {
        Self {
            components: Components::new(),
            forge: Forge::new(),
            executor: Executor::new(),
            observer: Observer::new(),
            bounds: Rectangle {
                x: Flint::new(50, 0),
                y: Flint::new(40, 0),
                width: Flint::new(700, 0),
                height: Flint::new(400, 0),
            },
            commands: Vec::new(),
            tracked_death: Vec::new(),
            tracked_movement: Vec::new(),
            dead: Vec::new(),
        }
    }

    pub fn update(&mut self, bus: &mut Bus, commands: &[Vec<Command>]) {
        self.execute_commands(bus, commands);

        self.executor.execute(
            &mut self.components,
            &mut self.forge,
            &self.bounds,
            &mut self.dead,
        );

        self.send_events(bus);
    }

    pub fn render(&self, renderer: &mut Renderer, toggle: bool, alpha: f32) {
        self.observer
            .observe(&self.components, renderer, &self.bounds, toggle, alpha);
    }

    pub fn event(&mut self, event: &Event) {
        let Event::Cosmos(events::Cosmos::Intent(event)) = event else {
            return;
        };

        match event {
            CosmosIntent::TrackDeath(entity) => self.tracked_death.push(*entity),
            CosmosIntent::TrackMovement(entity) => self.tracked_movement.push(*entity),
            CosmosIntent::Spawn { id, kind, centroid } => self.commands.push(Command::Spawn {
                id: *id,
                kind: *kind,
                centroid: *centroid,
            }),
        }
    }

    fn execute_commands(&mut self, bus: &mut Bus, commands: &[Vec<Command>]) {
        for command in self.commands.iter() {
            command.execute(&mut self.components, &mut self.forge, bus);
        }

        for command in commands.iter().flatten() {
            command.execute(&mut self.components, &mut self.forge, bus);
        }
    }

    fn send_events(&mut self, bus: &mut Bus) {
        for entity in self.dead.drain(..) {
            bus.send(CosmosEvent::Died(entity));

            for tracked_death in self.tracked_death.iter() {
                if entity == *tracked_death {
                    bus.send(CosmosEvent::TrackedDeath(entity));

                    // no need to keep tracking entities if they are dead
                    self.tracked_movement.retain_mut(|x| *x != entity);
                }
            }
        }

        for entity in self.tracked_movement.iter() {
            if let Some(body) = self.components.logic.bodies.get(&entity) {
                bus.send(CosmosEvent::TrackedMovement {
                    entity: *entity,
                    centroid: body.new.centroid,
                });
            }
        }
    }
}
