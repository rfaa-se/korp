use crate::{
    ecs::{commands::Command, components::Components},
    quadtree::Quadtree,
};
use korp_engine::{renderer::Renderer, shapes::Rectangle};
use korp_math::Flint;

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
        commands: &mut Vec<Command>,
        quadtree: &mut Quadtree,
    ) {
        use physics::*;

        morph_body(components);
        motion(components);
        hitbox(components);
        rebuild_quadtree(components, quadtree);
        collision(components, quadtree);
        out_of_bounds(bounds, components, commands);
        constant_accelerate(components, commands);

        morph_body_render(components);
        morph_hitbox_render(components);
        body_render(components);
        hitbox_render(components);
        quadtree_nodes_render(components, quadtree);
        cosmos_bounds_render(components, bounds);
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
        toggle: bool,
        alpha: f32,
    ) {
        use render::*;

        cosmos_bounds(components, renderer);
        quadtree_nodes(components, renderer);
        body(components, renderer, toggle, alpha);
        hitbox(components, renderer, alpha);
    }
}
