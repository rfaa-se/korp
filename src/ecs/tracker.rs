use crate::{
    bus::{Bus, events::CosmosEvent},
    ecs::{components::Components, entities::Entity},
};

#[derive(Debug)]
pub enum Track {
    Death(Entity),
    Movement(Entity),
}

pub struct Tracker {
    death: Vec<Entity>,
    movement: Vec<Entity>,
}

impl Tracker {
    pub fn new() -> Self {
        Self {
            death: Vec::new(),
            movement: Vec::new(),
        }
    }

    pub fn update(&mut self, components: &Components, bus: &mut Bus) {
        for entity in self.movement.iter() {
            if let Some(body) = components.logic.bodies.get(&entity) {
                bus.send(CosmosEvent::TrackedMovement {
                    entity: *entity,
                    centroid: body.new.centroid,
                });
            }
        }
    }

    pub fn track(&mut self, track: &Track) {
        match track {
            Track::Death(entity) => {
                self.death.push(*entity);
            }
            Track::Movement(entity) => {
                self.movement.push(*entity);
            }
        }
    }

    pub fn death(&mut self, entity: &Entity, bus: &mut Bus) {
        self.death.retain_mut(|x| {
            if x == entity {
                bus.send(CosmosEvent::TrackedDeath(*entity));

                // no need to keep tracking movement if entity is dead
                self.movement.retain_mut(|x| x != entity);

                return false;
            }

            true
        });
    }
}
