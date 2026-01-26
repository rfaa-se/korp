use korp_engine::{
    misc::Morph,
    renderer::Renderer,
    shapes::{Rectangle, Triangle},
};
use korp_math::{Vec2, lerp, lerp_angle};

use crate::ecs::components::{Body, Shape};

pub trait Renderable {
    fn render(&self, renderer: &mut Renderer, toggle: bool, alpha: f32);
}

impl Renderable for Morph<Body<f32>> {
    fn render(&self, renderer: &mut Renderer, toggle: bool, alpha: f32) {
        let rotation = Vec2::from_angle(lerp_angle(
            self.old.rotation.angle(),
            self.new.rotation.angle(),
            alpha,
        ));

        let centroid = Vec2::new(
            lerp(self.old.centroid.x, self.new.centroid.x, alpha),
            lerp(self.old.centroid.y, self.new.centroid.y, alpha),
        );

        // TODO: lerp color?
        let color = self.new.color;

        match (self.old.shape, self.new.shape) {
            (Shape::Triangle(old), Shape::Triangle(new)) => {
                let top = Vec2::new(
                    lerp(old.top.x, new.top.x, alpha),
                    lerp(old.top.y, new.top.y, alpha),
                );

                let left = Vec2::new(
                    lerp(old.left.x, new.left.x, alpha),
                    lerp(old.left.y, new.left.y, alpha),
                );

                let right = Vec2::new(
                    lerp(old.right.x, new.right.x, alpha),
                    lerp(old.right.y, new.right.y, alpha),
                );

                let shape = Triangle::from(top, left, right, centroid);

                if toggle {
                    renderer.draw_triangle_lines(shape, rotation, centroid, color);
                } else {
                    renderer.draw_triangle_filled(shape, rotation, centroid, color);
                }
            }
            (Shape::Rectangle(old), Shape::Rectangle(new)) => {
                let width = lerp(old.width, new.width, alpha);
                let height = lerp(old.height, new.height, alpha);
                let shape = Rectangle::from(width, height, centroid);

                if toggle {
                    renderer.draw_rectangle_lines(shape, rotation, centroid, color);
                } else {
                    renderer.draw_rectangle_filled(shape, rotation, centroid, color);
                }
            }
            // TODO: can't currently morph between different shapes, draw old or new?
            _ => panic!("wtf drawable shapes"),
        }
    }
}
