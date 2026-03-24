use crate::{
    bus::{Bus, events::CosmosEvent},
    ecs::{commands::Command, components::Components, cosmos::Configuration, tracker::Tracker},
    quadtree::Quadtree,
};
use korp_engine::{renderer::Renderer, shapes::Rectangle};
use korp_math::Flint;

mod physics;
mod render;

pub use physics::COSMIC_DRAG;

pub struct Executor {}
pub struct Observer {}
pub struct Processor {}

impl Executor {
    pub fn new() -> Self {
        Self {}
    }

    pub fn execute(
        &mut self,
        cosmos_bounds: Rectangle<Flint>,
        components: &mut Components,
        commands: &mut Vec<Command>,
        quadtree: &mut Quadtree,
        events: &mut Vec<CosmosEvent>,
    ) {
        use physics::*;

        morph_body(components);
        motion(components);
        hitbox(components);
        rebuild_quadtree(components, quadtree);
        collision(components, quadtree, events);
        out_of_cosmos_bounds(cosmos_bounds, components, commands);
        constant_accelerate(components, commands);

        morph_body_render(components);
        morph_hitbox_render(components);
        body_render(components);
        hitbox_render(components);
        quadtree_nodes_render(components, quadtree);
        cosmos_bounds_render(components, cosmos_bounds);
    }
}

impl Observer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn observe(
        &self,
        components: &Components,
        configuration: &Configuration,
        renderer: &mut Renderer,
        alpha: f32,
    ) {
        use render::*;

        cosmos_bounds(components, renderer);

        if configuration.draw_quadtree {
            quadtree_nodes(components, renderer);
        }

        body(components, renderer, configuration.draw_filled, alpha);

        if configuration.draw_hitbox {
            hitbox(components, renderer, alpha);
        }
    }
}

impl Processor {
    pub fn new() -> Self {
        Self {}
    }

    pub fn process(
        &self,
        _components: &mut Components,
        tracker: &mut Tracker,
        events: &mut Vec<CosmosEvent>,
        bus: &mut Bus,
    ) {
        for event in events.drain(..) {
            match event {
                CosmosEvent::Died(entity) => {
                    tracker.death(&entity, bus);
                }
                CosmosEvent::Collided {
                    alpha: _,
                    beta: _,
                    mtv: _,
                } => {
                    // TODO: use the minimum translation vector to push entities apart?
                }
                _ => (),
            }

            bus.send(event);
        }
    }
}
