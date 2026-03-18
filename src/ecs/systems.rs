use crate::ecs::{commands::Command, components::Components, entities::Entity, forge::Forge};
use korp_engine::{renderer::Renderer, shapes::Rectangle};
use korp_math::Flint;

mod brains;
mod physics;
mod render;

pub use physics::COSMIC_DRAG;

pub struct Executor {}
pub struct Observer {}

impl Executor {
    pub fn new() -> Self {
        Self {}
    }

    pub fn execute(
        &mut self,
        bounds: &Rectangle<Flint>,
        components: &mut Components,
        forge: &mut Forge,
        dead: &mut Vec<Entity>,
        commands: &mut Vec<Command>,
    ) {
        use brains::*;
        use physics::*;

        morph_body(components);
        motion(components);
        hitbox(components);
        collision(components);
        out_of_bounds(bounds, forge, components, dead);

        morph_body_render(components);
        morph_hitbox_render(components);
        body_render(components);
        hitbox_render(components);

        projectiles(components, commands);
    }
}

impl Observer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn observe(
        &self,
        components: &Components,
        renderer: &mut Renderer,
        bounds: &Rectangle<Flint>,
        toggle: bool,
        alpha: f32,
    ) {
        use render::*;

        cosmos(bounds, renderer);
        body(components, renderer, toggle, alpha);
        hitbox(components, renderer, alpha);
    }
}
