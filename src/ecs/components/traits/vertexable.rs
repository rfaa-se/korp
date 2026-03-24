use korp_math::{Flint, Vec2};

use crate::ecs::components::{Body, Shape};

pub trait Vertexable {
    fn vertices(&self) -> Vec<Vec2<Flint>>;
}

impl Vertexable for Body<Flint> {
    fn vertices(&self) -> Vec<Vec2<Flint>> {
        match self.shape {
            Shape::Triangle(triangle) => {
                let t = self.centroid + triangle.top.rotated_v(self.rotation);
                let l = self.centroid + triangle.left.rotated_v(self.rotation);
                let r = self.centroid + triangle.right.rotated_v(self.rotation);

                vec![t, l, r]
            }
            Shape::Rectangle(rectangle) => {
                let w = rectangle.width * Flint::ZERO_FIVE;
                let h = rectangle.height * Flint::ZERO_FIVE;

                let tl = self.centroid + Vec2::new(-w, -h).rotated_v(self.rotation);
                let tr = self.centroid + Vec2::new(w, -h).rotated_v(self.rotation);
                let bl = self.centroid + Vec2::new(-w, h).rotated_v(self.rotation);
                let br = self.centroid + Vec2::new(w, h).rotated_v(self.rotation);

                vec![tl, tr, bl, br]
            }
        }
    }
}
