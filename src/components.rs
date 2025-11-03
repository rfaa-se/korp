use korp_engine::color::Color;
use korp_math::Vec2;

pub mod traits;

#[derive(Copy, Clone)]
pub struct Body {
    pub centroid: Vec2<f32>,
    pub rotation: Vec2<f32>,
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
    pub top: Vec2<f32>,
    pub left: Vec2<f32>,
    pub right: Vec2<f32>,
}

#[derive(Copy, Clone)]
pub struct Rectangle {
    pub width: f32,
    pub height: f32,
}
