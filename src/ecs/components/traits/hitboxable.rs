use korp_engine::{misc::Morph, shapes::Rectangle};
use korp_math::{Flint, Vec2};

use crate::ecs::components::{Body, Shape};

pub trait Hitboxable {
    fn hitbox(&self) -> Rectangle<Flint>;
}

impl Hitboxable for Body {
    fn hitbox(&self) -> Rectangle<Flint> {
        match self.shape {
            Shape::Triangle(triangle) => {
                let t = self.centroid + triangle.top.rotated_v(self.rotation);
                let l = self.centroid + triangle.left.rotated_v(self.rotation);
                let r = self.centroid + triangle.right.rotated_v(self.rotation);

                let xmin = t.x.min(l.x.min(r.x));
                let xmax = t.x.max(l.x.max(r.x));
                let ymin = t.y.min(l.y.min(r.y));
                let ymax = t.y.max(l.y.max(r.y));

                Rectangle {
                    x: xmin,
                    y: ymin,
                    width: xmax - xmin,
                    height: ymax - ymin,
                }
            }
            Shape::Rectangle(rectangle) => {
                let w = rectangle.width * Flint::ZERO_FIVE;
                let h = rectangle.height * Flint::ZERO_FIVE;

                let tl = self.centroid + Vec2::new(-w, -h).rotated_v(self.rotation);
                let tr = self.centroid + Vec2::new(w, -h).rotated_v(self.rotation);
                let bl = self.centroid + Vec2::new(-w, h).rotated_v(self.rotation);
                let br = self.centroid + Vec2::new(w, h).rotated_v(self.rotation);

                let xmin = tl.x.min(tr.x.min(bl.x.min(br.x)));
                let xmax = tl.x.max(tr.x.max(bl.x.max(br.x)));
                let ymin = tl.y.min(tr.y.min(bl.y.min(br.y)));
                let ymax = tl.y.max(tr.y.max(bl.y.max(br.y)));

                Rectangle {
                    x: xmin,
                    y: ymin,
                    width: xmax - xmin,
                    height: ymax - ymin,
                }
            }
        }
    }
}

impl Hitboxable for Morph<Body> {
    fn hitbox(&self) -> Rectangle<Flint> {
        match (self.old.shape, self.new.shape) {
            (Shape::Triangle(_), Shape::Triangle(_)) => {
                let old = self.old.hitbox();
                let new = self.new.hitbox();
                let xmin = old.x.min(new.x);
                let xmax = (old.x + old.width).max(new.x + new.width);
                let ymin = old.y.min(new.y);
                let ymax = (old.y + old.height).max(new.y + new.height);

                Rectangle {
                    x: xmin,
                    y: ymin,
                    width: xmax - xmin,
                    height: ymax - ymin,
                }
            }
            (Shape::Rectangle(_), Shape::Rectangle(_)) => {
                let old = self.old.hitbox();
                let new = self.new.hitbox();
                let xmin = old.x.min(new.x);
                let xmax = (old.x + old.width).max(new.x + new.width);
                let ymin = old.y.min(new.y);
                let ymax = (old.y + old.height).max(new.y + new.height);

                Rectangle {
                    x: xmin,
                    y: ymin,
                    width: xmax - xmin,
                    height: ymax - ymin,
                }
            }
            _ => panic!("wtf hitboxable shapes"),
        }
    }
}
