use korp_engine::{misc::Morph, shapes::Rectangle};
use korp_math::Flint;

use crate::ecs::components::{Body, Shape, traits::Vertexable};

pub trait Hitboxable {
    fn hitbox(&self) -> Rectangle<Flint>;
}

impl Hitboxable for Body<Flint> {
    fn hitbox(&self) -> Rectangle<Flint> {
        let vertices = self.vertices();

        match self.shape {
            Shape::Triangle(_) => {
                let t = vertices[0];
                let l = vertices[1];
                let r = vertices[2];

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
            Shape::Rectangle(_) => {
                let tl = vertices[0];
                let tr = vertices[1];
                let bl = vertices[2];
                let br = vertices[3];

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

impl Hitboxable for Morph<Body<Flint>> {
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
