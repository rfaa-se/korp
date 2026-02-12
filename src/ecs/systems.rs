use crate::ecs::{components::Components, entities::Entity, forge::Forge};
use korp_engine::{renderer::Renderer, shapes::Rectangle};
use korp_math::Flint;

mod physics;
mod render;

pub struct Executor {}
pub struct Observer {}

impl Executor {
    pub fn new() -> Self {
        Self {}
    }

    pub fn execute(
        &mut self,
        components: &mut Components,
        forge: &mut Forge,
        bounds: &Rectangle<Flint>,
        dead: &mut Vec<Entity>,
    ) {
        use physics::*;

        morph_body(components);
        motion(components);
        hitbox(components);
        out_of_bounds(bounds, forge, components, dead);

        morph_body_render(components);
        morph_hitbox_render(components);
        body_render(components);
        hitbox_render(components);
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
