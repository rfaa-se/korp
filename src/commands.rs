use korp_math::{Flint, Vec2};

use crate::{
    bus::{Bus, events::CosmosEvent},
    ecs::{components::Components, entities::Entity, forge::Forge},
};

#[derive(Debug, Clone)]
pub enum Command {
    Accelerate(Entity),
    Decelerate(Entity),
    TurnLeft(Entity),
    TurnRight(Entity),
    Spawn {
        kind: SpawnKind,
        centroid: Vec2<Flint>,
    },
}

#[derive(Debug, Clone)]
pub enum SpawnKind {
    Triangle,
    Rectangle,
}

impl Command {
    pub fn execute(&self, components: &mut Components, forge: &mut Forge, bus: &mut Bus) {
        match self {
            Command::Accelerate(entity) => handle_accelerate(entity, components),
            Command::Decelerate(entity) => handle_decelerate(entity, components),
            Command::TurnLeft(entity) => handle_turn_left(entity, components),
            Command::TurnRight(entity) => handle_turn_right(entity, components),
            Command::Spawn { kind, centroid } => {
                handle_spawn(kind, centroid, components, forge, bus)
            }
        }
    }
}

fn handle_accelerate(entity: &Entity, components: &mut Components) {
    let (Some(motion), Some(body)) = (
        components.logic.motions.get_mut(entity),
        components.logic.bodies.get(entity),
    ) else {
        return;
    };

    motion.velocity += body.new.rotation * motion.acceleration;
}

fn handle_decelerate(entity: &Entity, components: &mut Components) {
    let (Some(motion), Some(body)) = (
        components.logic.motions.get_mut(entity),
        components.logic.bodies.get(entity),
    ) else {
        return;
    };

    motion.velocity -= body.new.rotation * motion.acceleration;
}

fn handle_turn_left(entity: &Entity, components: &mut Components) {
    let Some(motion) = components.logic.motions.get_mut(entity) else {
        return;
    };

    motion.rotation_speed -= motion.rotation_acceleration;
}

fn handle_turn_right(entity: &Entity, components: &mut Components) {
    let Some(motion) = components.logic.motions.get_mut(entity) else {
        return;
    };

    motion.rotation_speed += motion.rotation_acceleration;
}

fn handle_spawn(
    kind: &SpawnKind,
    centroid: &Vec2<Flint>,
    components: &mut Components,
    forge: &mut Forge,
    bus: &mut Bus,
) {
    let entity = match kind {
        SpawnKind::Triangle => forge.triangle(*centroid, components),
        SpawnKind::Rectangle => forge.rectangle(*centroid, components),
    };

    bus.send(CosmosEvent::Spawned(entity));
}
