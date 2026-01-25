use crate::{
    commands::Command,
    ecs::{
        components::{Components, traits::Renderable},
        forge::Forge,
    },
};
use korp_engine::{color::Color, renderer::Renderer, shapes::Rectangle};
use korp_math::{Flint, Vec2, lerp};

mod physics;

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
        commands: &[Command],
    ) {
        use physics::*;

        morph_body(components);
        execute_commands(components, forge, commands);
        motion(components);
        hitbox(components);
        out_of_bounds(bounds, forge, components);
        morph_hitbox_render(components);
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
        self.cosmos(bounds, renderer);

        for (_, body) in components.logic.bodies.iter() {
            body.render(renderer, toggle, alpha);
        }

        for (_, hitbox) in components.render.hitboxes.iter() {
            let width = lerp(hitbox.old.width, hitbox.new.width, alpha);
            let height = lerp(hitbox.old.height, hitbox.new.height, alpha);
            let centroid = Vec2::new(
                lerp(hitbox.old.x, hitbox.new.x, alpha) + width * 0.5,
                lerp(hitbox.old.y, hitbox.new.y, alpha) + height * 0.5,
            );

            let rectangle = Rectangle::from(width, height, centroid);

            renderer.draw_rectangle_lines(rectangle, Vec2::new(1.0, 0.0), centroid, Color::BLUE);
        }
    }

    fn cosmos(&self, dimensions: &Rectangle<Flint>, renderer: &mut Renderer) {
        let dimensions = Rectangle {
            x: dimensions.x.to_f32(),
            y: dimensions.y.to_f32(),
            width: dimensions.width.to_f32(),
            height: dimensions.height.to_f32(),
        };

        let rotation = Vec2::new(1.0, 0.0);

        let origin = Vec2::new(
            dimensions.x + dimensions.width * 0.5,
            dimensions.y + dimensions.height * 0.5,
        );

        let color = Color::RED;

        renderer.draw_rectangle_lines(dimensions, rotation, origin, color);
    }
}

pub fn execute_commands(components: &mut Components, forge: &mut Forge, commands: &[Command]) {
    for command in commands {
        command.execute(components, forge);
    }
}
