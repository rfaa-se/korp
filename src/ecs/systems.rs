use crate::{
    commands::Command,
    ecs::{
        components::{
            Components,
            traits::{Drawable, Hitboxable},
        },
        forge::Forge,
    },
};
use korp_engine::{color::Color, misc::Morph, renderer::Canvas, shapes::Rectangle};
use korp_math::{Flint, Vec2};

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
        out_of_bounds(bounds, forge, components);
    }
}

impl Observer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn observe(
        &self,
        components: &Components,
        canvas: &mut Canvas,
        bounds: &Rectangle<Flint>,
        toggle: bool,
    ) {
        self.cosmos(bounds, canvas);

        for (_, body) in components.bodies.iter() {
            body.draw(canvas, toggle);

            let old = body.old.hitbox();
            let new = body.new.hitbox();

            canvas.draw_rectangle_lines(
                Morph::new(old.into(), new.into()),
                Morph::one(Vec2::new(1.0, 0.0)),
                Morph::new(body.old.centroid.into(), body.new.centroid.into()),
                Morph::one(Color::BLUE),
            );
        }
    }

    fn cosmos(&self, dimensions: &Rectangle<Flint>, canvas: &mut Canvas) {
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

        canvas.draw_rectangle_lines(
            Morph::one(dimensions),
            Morph::one(rotation),
            Morph::one(origin),
            Morph::one(color),
        );
    }
}

pub fn execute_commands(components: &mut Components, forge: &mut Forge, commands: &[Command]) {
    for command in commands {
        command.execute(components, forge);
    }
}
