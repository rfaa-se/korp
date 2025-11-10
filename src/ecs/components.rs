use korp_engine::color::Color;
use korp_math::{Flint, Vec2};

pub mod traits;

pub struct Motion {
    pub velocity: Vec2<Flint>,
    pub speed_maximum: Flint,
    pub speed_minimum: Flint,
    pub acceleration: Flint,
    pub rotation_speed: Flint,
    pub rotation_speed_maximum: Flint,
    pub rotation_speed_minimum: Flint,
    pub rotation_acceleration: Flint,
}

#[derive(Copy, Clone)]
pub struct Body {
    pub centroid: Vec2<Flint>,
    pub rotation: Vec2<Flint>,
    pub shape: Shape,
    pub color: Color,
}

#[derive(Copy, Clone)]
pub enum Shape {
    Triangle(Triangle),
    Rectangle(Rectangle),
}

#[derive(Copy, Clone)]
pub struct Triangle {
    pub top: Vec2<Flint>,
    pub left: Vec2<Flint>,
    pub right: Vec2<Flint>,
}

#[derive(Copy, Clone)]
pub struct Rectangle {
    pub width: Flint,
    pub height: Flint,
}
