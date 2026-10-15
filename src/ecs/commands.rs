use korp_math::{Flint, Random, Vec2};

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
    RotateLeft(Entity),
    RotateRight(Entity),
    Shoot(Entity),
    Kill(Entity),
    Spawn { id: Option<usize>, kind: SpawnKind },
}

#[derive(Debug, Clone, Copy)]
pub enum SpawnKind {
    Triangle {
        centroid: Vec2<Flint>,
    },
    Rectangle {
        centroid: Vec2<Flint>,
    },
    Projectile {
        owner: Entity,
        relative_speed: Flint,
        centroid: Vec2<Flint>,
        rotation: Vec2<Flint>,
    },
    Particle {
        centroid: Vec2<Flint>,
        direction: Vec2<Flint>,
        speed: Flint,
        lifetime: u32,
    },
}

impl Command {
    pub fn execute(
        &self,
        components: &mut Components,
        forge: &mut Forge,
        random: &mut Random,
        events: &mut Vec<CosmosEvent>,
    ) {
        match self {
            Command::Accelerate(entity) => accelerate(*entity, components),
            Command::Decelerate(entity) => decelerate(*entity, components),
            Command::RotateLeft(entity) => rotate_left(*entity, components),
            Command::RotateRight(entity) => rotate_right(*entity, components),
            Command::Shoot(entity) => shoot(*entity, components, forge, events),
            Command::Kill(entity) => kill(*entity, components, forge, random, events),
            Command::Spawn { id, kind } => spawn(id, kind, components, forge, events),
        }
    }
}

fn kill(
    entity: Entity,
    components: &mut Components,
    forge: &mut Forge,
    random: &mut Random,
    events: &mut Vec<CosmosEvent>,
) {
    if forge.destroy(entity) {
        if let Some(body) = components.logic.bodies.remove(entity)
            && components.logic.death_explosives.get(entity).is_some()
        {
            forge.explode(body.new, random, components);
        }

        components.destroy(entity);
        events.push(CosmosEvent::Died(entity));
    }
}

fn accelerate(entity: Entity, components: &mut Components) {
    let (Some(motion), Some(body)) = (
        components.logic.motions.get_mut(entity),
        components.logic.bodies.get(entity),
    ) else {
        return;
    };

    motion.velocity += body.new.rotation * motion.acceleration;

    if let Some(emitter) = components.logic.exhaust_emitters.get_mut(entity)
        && emitter.age < emitter.lifetime
    {
        emitter.age += 1;
    }
}

fn decelerate(entity: Entity, components: &mut Components) {
    let (Some(motion), Some(body)) = (
        components.logic.motions.get_mut(entity),
        components.logic.bodies.get(entity),
    ) else {
        return;
    };

    let speed = motion.velocity.dot(&body.new.rotation);
    if speed <= motion.speed_minimum {
        return;
    }

    motion.velocity -= body.new.rotation * motion.acceleration;

    let speed = motion.velocity.dot(&body.new.rotation);
    if speed < motion.speed_minimum {
        motion.velocity += body.new.rotation * (motion.speed_minimum - speed);
    }
}

fn rotate_left(entity: Entity, components: &mut Components) {
    let Some(motion) = components.logic.motions.get_mut(entity) else {
        return;
    };

    motion.rotation_speed -= motion.rotation_acceleration;
}

fn rotate_right(entity: Entity, components: &mut Components) {
    let Some(motion) = components.logic.motions.get_mut(entity) else {
        return;
    };

    motion.rotation_speed += motion.rotation_acceleration;
}

fn shoot(
    entity: Entity,
    components: &mut Components,
    forge: &mut Forge,
    events: &mut Vec<CosmosEvent>,
) {
    let Some(body) = components.logic.bodies.get(entity) else {
        return;
    };

    // calculate the spawn point
    let rotation = body.new.rotation;
    let centroid = body.new.centroid
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

    spawn(
        &None,
        &SpawnKind::Projectile {
            owner: entity,
            relative_speed,
            centroid,
            rotation,
        },
        components,
        forge,
        events,
    );
}

fn spawn(
    id: &Option<usize>,
    kind: &SpawnKind,
    components: &mut Components,
    forge: &mut Forge,
    events: &mut Vec<CosmosEvent>,
) {
    let entity = match kind {
        SpawnKind::Triangle { centroid } => forge.triangle(*centroid, components),
        SpawnKind::Rectangle { centroid } => forge.rectangle(*centroid, components),
        SpawnKind::Projectile {
            owner,
            relative_speed,
            centroid,
            rotation,
        } => forge.projectile(*owner, *relative_speed, *centroid, *rotation, components),
        SpawnKind::Particle {
            centroid,
            direction,
            speed,
            lifetime,
        } => {
            // particles are not entities
            forge.particle(*centroid, *direction, *speed, *lifetime, components);
            return;
        }
    };

    events.push(CosmosEvent::Spawned { id: *id, entity });
}
