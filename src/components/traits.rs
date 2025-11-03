use korp_engine::{
    misc::Morph,
    renderer::Canvas,
    shapes::{Rectangle, Triangle},
};

use crate::components::{Body, Shape};

pub trait Drawable {
    fn draw(&self, canvas: &mut Canvas, toggle: bool);
}

impl Drawable for Morph<Body> {
    fn draw(&self, canvas: &mut Canvas, toggle: bool) {
        let rotation = Morph {
            old: self.old.rotation,
            new: self.new.rotation,
        };

        let centroid = Morph {
            old: self.old.centroid,
            new: self.new.centroid,
        };

        let color = Morph {
            old: self.old.color,
            new: self.new.color,
        };

        match (self.old.shape, self.new.shape) {
            (Shape::Triangle(old), Shape::Triangle(new)) => {
                if toggle {
                    canvas.draw_triangle_lines(
                        Morph {
                            old: Triangle::from(old.top, old.left, old.right, self.old.centroid),
                            new: Triangle::from(new.top, new.left, new.right, self.new.centroid),
                        },
                        rotation,
                        centroid,
                        color,
                    );
                } else {
                    canvas.draw_triangle_filled(
                        Morph {
                            old: Triangle::from(old.top, old.left, old.right, self.old.centroid),
                            new: Triangle::from(new.top, new.left, new.right, self.new.centroid),
                        },
                        rotation,
                        centroid,
                        color,
                    );
                }
            }
            (Shape::Rectangle(old), Shape::Rectangle(new)) => {
                if toggle {
                    canvas.draw_rectangle_lines(
                        Morph {
                            old: Rectangle::from(old.width, old.height, self.old.centroid),
                            new: Rectangle::from(new.width, new.height, self.new.centroid),
                        },
                        rotation,
                        centroid,
                        color,
                    );
                } else {
                    canvas.draw_rectangle_filled(
                        Morph {
                            old: Rectangle::from(old.width, old.height, self.old.centroid),
                            new: Rectangle::from(new.width, new.height, self.new.centroid),
                        },
                        rotation,
                        centroid,
                        color,
                    );
                }
            }
            // TODO: can't morph between different shapes, draw old or new?
            _ => (),
        }
    }
}
