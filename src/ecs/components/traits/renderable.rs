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

impl Renderable for Morph<Body> {
    fn render(&self, renderer: &mut Renderer, toggle: bool, alpha: f32) {
        let rot_old: Vec2<f32> = self.old.rotation.into();
        let rot_new: Vec2<f32> = self.new.rotation.into();
        let rotation = lerp_angle(rot_old.angle(), rot_new.angle(), alpha);
        let rotation = Vec2::from_angle(rotation);

        let cen_old: Vec2<f32> = self.old.centroid.into();
        let cen_new: Vec2<f32> = self.new.centroid.into();
        let centroid = Vec2::new(
            lerp(cen_old.x, cen_new.x, alpha),
            lerp(cen_old.y, cen_new.y, alpha),
        );

        // TODO: lerp color?
        let color = self.new.color;

        match (self.old.shape, self.new.shape) {
            (Shape::Triangle(old), Shape::Triangle(new)) => {
                let top = Vec2::new(
                    lerp(old.top.x.into(), new.top.x.into(), alpha),
                    lerp(old.top.y.into(), new.top.y.into(), alpha),
                );

                let left = Vec2::new(
                    lerp(old.left.x.into(), new.left.x.into(), alpha),
                    lerp(old.left.y.into(), new.left.y.into(), alpha),
                );

                let right = Vec2::new(
                    lerp(old.right.x.into(), new.right.x.into(), alpha),
                    lerp(old.right.y.into(), new.right.y.into(), alpha),
                );

                let shape = Triangle::from(top, left, right, centroid);

                if toggle {
                    renderer.draw_triangle_lines(shape, rotation, centroid, color);
                } else {
                    renderer.draw_triangle_filled(shape, rotation, centroid, color);
                }
            }
            (Shape::Rectangle(old), Shape::Rectangle(new)) => {
                let width = lerp(old.width.into(), new.width.into(), alpha);
                let height = lerp(old.height.into(), new.height.into(), alpha);
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
