use korp_engine::misc::Morph;
use korp_math::Flint;

use crate::ecs::components::{Body, Rectangle, Shape, Triangle};

pub trait Transformable<T> {
    fn transform(&self) -> T;
}

impl Transformable<Morph<Body<f32>>> for Morph<Body<Flint>> {
    fn transform(&self) -> Morph<Body<f32>> {
        Morph::new(
            Body {
                centroid: self.old.centroid.into(),
                rotation: self.old.rotation.into(),
                shape: self.old.shape.transform(),
                color: self.old.color,
            },
            Body {
                centroid: self.new.centroid.into(),
                rotation: self.new.rotation.into(),
                shape: self.new.shape.transform(),
                color: self.new.color,
            },
        )
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
