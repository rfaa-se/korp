use korp_math::{Flint, Vec2};

use crate::{
    bus::{Bus, events::CosmosEvent},
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
    pub fn execute(&self, components: &mut Components, forge: &mut Forge, bus: &mut Bus) {
        match self {
            Command::Accelerate(entity) => accelerate(entity, components),
            Command::Decelerate(entity) => decelerate(entity, components),
            Command::TurnLeft(entity) => turn_left(entity, components),
            Command::TurnRight(entity) => turn_right(entity, components),
            Command::Shoot(entity) => shoot(entity, components, forge, bus),
            Command::Spawn { id, kind, centroid } => {
                spawn(id, kind, centroid, components, forge, bus)
            }
        }
    }
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

fn shoot(entity: &Entity, components: &mut Components, forge: &mut Forge, bus: &mut Bus) {
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

    let entity = forge.projectile(point, rotation, components);

    bus.send(CosmosEvent::Spawned { id: None, entity });
}

fn spawn(
    id: &Option<usize>,
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

    bus.send(CosmosEvent::Spawned { id: *id, entity });
}
