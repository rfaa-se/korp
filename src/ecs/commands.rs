use korp_math::{Flint, Vec2};

use crate::{
    bus::events::CosmosEvent,
    ecs::{
        components::{Components, Shape},
        entities::Entity,
        forge::Forge,
    },
};

#[derive(Debug, Clone)]
pub enum Command {
    Accelerate(Entity),
    Decelerate(Entity),
    TurnLeft(Entity),
    TurnRight(Entity),
    Shoot(Entity),
    Kill(Entity),
    Spawn {
        id: Option<usize>,
        kind: SpawnKind,
        centroid: Vec2<Flint>,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum SpawnKind {
    Triangle,
    Rectangle,
}

impl Command {
    pub fn execute(
        &self,
        components: &mut Components,
        forge: &mut Forge,
        events: &mut Vec<CosmosEvent>,
    ) {
        match self {
            Command::Accelerate(entity) => accelerate(entity, components),
            Command::Decelerate(entity) => decelerate(entity, components),
            Command::TurnLeft(entity) => turn_left(entity, components),
            Command::TurnRight(entity) => turn_right(entity, components),
            Command::Shoot(entity) => shoot(entity, components, forge, events),
            Command::Kill(entity) => kill(entity, components, forge, events),
            Command::Spawn { id, kind, centroid } => {
                spawn(id, kind, centroid, components, forge, events)
            }
        }
    }
}

fn kill(
    entity: &Entity,
    components: &mut Components,
    forge: &mut Forge,
    events: &mut Vec<CosmosEvent>,
) {
    forge.destroy(*entity, components);
    events.push(CosmosEvent::Died(*entity));
}

fn accelerate(entity: &Entity, components: &mut Components) {
    let (Some(motion), Some(body)) = (
        components.logic.motions.get_mut(entity),
        components.logic.bodies.get(entity),
    ) else {
        return;
    };

    motion.velocity += body.new.rotation * motion.acceleration;
}

fn decelerate(entity: &Entity, components: &mut Components) {
    let (Some(motion), Some(body)) = (
        components.logic.motions.get_mut(entity),
        components.logic.bodies.get(entity),
    ) else {
        return;
    };

    motion.velocity -= body.new.rotation * motion.acceleration;
}

fn turn_left(entity: &Entity, components: &mut Components) {
    let Some(motion) = components.logic.motions.get_mut(entity) else {
        return;
    };

    motion.rotation_speed -= motion.rotation_acceleration;
}

fn turn_right(entity: &Entity, components: &mut Components) {
    let Some(motion) = components.logic.motions.get_mut(entity) else {
        return;
    };

    motion.rotation_speed += motion.rotation_acceleration;
}

fn shoot(
    entity: &Entity,
    components: &mut Components,
    forge: &mut Forge,
    events: &mut Vec<CosmosEvent>,
) {
    let Some(body) = components.logic.bodies.get(entity) else {
        return;
    };

    // calculate the spawn point
    let rotation = body.new.rotation;
    let point = body.new.centroid
        + match body.new.shape {
            Shape::Triangle(triangle) => triangle.top,
            Shape::Rectangle(rectangle) => {
                Vec2::new(rectangle.width * Flint::ZERO_FIVE, rectangle.height)
            }
        }
        .rotated_v(rotation);

    let relative_speed = match components.logic.motions.get(entity) {
        Some(motion) => motion.velocity.len(),
        None => Flint::ZERO,
    };

    let entity = forge.projectile(*entity, relative_speed, point, rotation, components);

    events.push(CosmosEvent::Spawned { id: None, entity });
}

fn spawn(
    id: &Option<usize>,
    kind: &SpawnKind,
    centroid: &Vec2<Flint>,
    components: &mut Components,
    forge: &mut Forge,
    events: &mut Vec<CosmosEvent>,
) {
    let entity = match kind {
        SpawnKind::Triangle => forge.triangle(*centroid, components),
        SpawnKind::Rectangle => forge.rectangle(*centroid, components),
    };

    events.push(CosmosEvent::Spawned { id: *id, entity });
}
