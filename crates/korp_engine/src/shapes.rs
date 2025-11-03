use korp_math::Vec2;

#[derive(Copy, Clone)]
pub struct Line {
    pub start: Vec2<f32>,
    pub end: Vec2<f32>,
}

#[derive(Copy, Clone)]
pub struct Triangle {
    pub top: Vec2<f32>,
    pub left: Vec2<f32>,
    pub right: Vec2<f32>,
}

#[derive(Copy, Clone)]
pub struct Rectangle {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Triangle {
    pub fn from(
        top: Vec2<f32>,
        left: Vec2<f32>,
        right: Vec2<f32>,
        centroid: Vec2<f32>,
    ) -> Triangle {
        Triangle {
            top: centroid + top,
            left: centroid + left,
            right: centroid + right,
        }
    }
}

impl Rectangle {
    pub fn from(width: f32, height: f32, centroid: Vec2<f32>) -> Rectangle {
        Rectangle {
            x: centroid.x - width * 0.5,
            y: centroid.y - height * 0.5,
            width,
            height,
        }
    }
}
