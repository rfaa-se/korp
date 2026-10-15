use korp_engine::misc::Morph;
use korp_math::Flint;

use crate::ecs::components::{Body, Rectangle, Shape, Triangle};

pub trait Transformable<T> {
    fn transform(&self) -> T;
}

impl Transformable<Body<f32>> for Body<Flint> {
    fn transform(&self) -> Body<f32> {
        Body {
            centroid: self.centroid.into(),
            rotation: self.rotation.into(),
            shape: match self.shape {
                Shape::Triangle(triangle) => Shape::Triangle(Triangle {
                    top: triangle.top.into(),
                    left: triangle.left.into(),
                    right: triangle.right.into(),
                }),
                Shape::Rectangle(rectangle) => Shape::Rectangle(Rectangle {
                    width: rectangle.width.into(),
                    height: rectangle.height.into(),
                }),
            },
            color: self.color,
        }
    }
}

impl Transformable<Morph<Body<f32>>> for Morph<Body<Flint>> {
    fn transform(&self) -> Morph<Body<f32>> {
        Morph::new(self.old.transform(), self.new.transform())
    }
}

impl Transformable<Shape<f32>> for Shape<Flint> {
    fn transform(&self) -> Shape<f32> {
        match self {
            Shape::Triangle(triangle) => Shape::Triangle(Triangle {
                top: triangle.top.into(),
                left: triangle.left.into(),
                right: triangle.right.into(),
            }),
            Shape::Rectangle(rectangle) => Shape::Rectangle(Rectangle {
                width: rectangle.width.into(),
                height: rectangle.height.into(),
            }),
        }
    }
}
