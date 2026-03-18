use crate::ecs::{commands::Command, components::Components};

pub fn projectiles(components: &mut Components, commands: &mut Vec<Command>) {
    for (&entity, _) in components.logic.brains.projectiles.iter() {
        commands.push(Command::Accelerate(entity));
    }
}
