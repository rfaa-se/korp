use korp_math::{Flint, Vec2};

use crate::ecs::{components::Components, entities::Entity, forge::Forge};

pub enum Command {
    Accelerate(Entity),
    Decelerate(Entity),
    TurnLeft(Entity),
    TurnRight(Entity),
    Spawn(Spawn, Vec2<Flint>),
}

pub enum Spawn {
    Triangle,
    Rectangle,
}

impl Command {
    pub fn execute(&self, components: &mut Components, forge: &mut Forge) {
        match self {
            Command::Accelerate(entity) => handle_accelerate(entity, components),
            Command::Decelerate(entity) => handle_decelerate(entity, components),
            Command::TurnLeft(entity) => handle_turn_left(entity, components),
            Command::TurnRight(entity) => handle_turn_right(entity, components),
            Command::Spawn(spawn, centroid) => handle_spawn(spawn, centroid, components, forge),
        }
    }
}

fn handle_accelerate(entity: &Entity, components: &mut Components) {
    let (Some(motion), Some(body)) = (
        components.motions.get_mut(entity),
        components.bodies.get(entity),
    ) else {
        return;
    };

    motion.velocity += body.new.rotation * motion.acceleration;
}

fn handle_decelerate(entity: &Entity, components: &mut Components) {
    let (Some(motion), Some(body)) = (
        components.motions.get_mut(entity),
        components.bodies.get(entity),
    ) else {
        return;
    };

    motion.velocity -= body.new.rotation * motion.acceleration;
}

fn handle_turn_left(entity: &Entity, components: &mut Components) {
    let Some(motion) = components.motions.get_mut(entity) else {
        return;
    };

    motion.rotation_speed -= motion.rotation_acceleration;
}

fn handle_turn_right(entity: &Entity, components: &mut Components) {
    let Some(motion) = components.motions.get_mut(entity) else {
        return;
    };

    motion.rotation_speed += motion.rotation_acceleration;
}

fn handle_spawn(
    spawn: &Spawn,
    centroid: &Vec2<Flint>,
    components: &mut Components,
    forge: &mut Forge,
) {
    match spawn {
        Spawn::Triangle => {
            forge.triangle(*centroid, components);
        }
        Spawn::Rectangle => {
            forge.rectangle(*centroid, components);
        }
    }
}
