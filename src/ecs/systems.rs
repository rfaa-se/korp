use crate::{
    commands::Command,
    ecs::{components::traits::Drawable, cosmos::Components, forge::Forge},
};
use korp_engine::renderer::Canvas;

mod physics;

pub struct Executor {}
pub struct Renderer {}

impl Executor {
    pub fn new() -> Self {
        Self {}
    }

    pub fn execute(
        &mut self,
        components: &mut Components,
        forge: &mut Forge,
        commands: &[Command],
    ) {
        use physics::*;

        morph_body(components);
        execute_commands(components, forge, commands);
        motion(components);
    }
}

impl Renderer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render(&self, components: &Components, canvas: &mut Canvas, toggle: bool) {
        for (_, body) in components.bodies.iter() {
            body.draw(canvas, toggle);
        }
    }
}

pub fn execute_commands(components: &mut Components, forge: &mut Forge, commands: &[Command]) {
    for command in commands {
        command.execute(components, forge);
    }
}
