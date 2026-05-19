use korp_math::Random;

use crate::{
    bus::{Bus, events::CosmosEvent},
    ecs::{
        commands::Command, components::Components, entities::Entity, forge::Forge, tracker::Tracker,
    },
};

pub struct Processor {}

impl Processor {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(
        &self,
        random: &mut Random,
        components: &mut Components,
        graveyard: &mut Components,
        tracker: &mut Tracker,
        events: &mut Vec<CosmosEvent>,
        commands: &mut Vec<Command>,
        forge: &mut Forge,
        bus: &mut Bus,
    ) {
        for event in events.drain(..) {
            match event {
                CosmosEvent::Died(entity) => {
                    if graveyard.logic.death_explosives.get(&entity).is_some() {
                        forge.explode(entity, random, components, graveyard);
                    }

                    tracker.death(&entity, bus);
                    // TODO: save dead entity, remove from graveyard next tick
                }
                CosmosEvent::Collided {
                    alpha,
                    beta,
                    mtv: _,
                } => {
                    // TODO: use the minimum translation vector to push entities apart?
                    if spawn_protected(alpha, beta, components) {
                        continue;
                    }

                    commands.push(Command::Kill(alpha));
                    commands.push(Command::Kill(beta));
                }
                _ => (),
            }

            bus.send(event);
        }
    }
}

fn spawn_protected(a: Entity, b: Entity, components: &Components) -> bool {
    let protected = |a, b| {
        let Some(_) = components.logic.spawn_protections.get(&a) else {
            return false;
        };

        let Some(owner) = components.logic.owners.get(&a) else {
            return false;
        };

        if owner.entity != b {
            return false;
        }

        true
    };

    protected(a, b) || protected(b, a)
}
