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
        top: impl Into<Vec2<f32>>,
        left: impl Into<Vec2<f32>>,
        right: impl Into<Vec2<f32>>,
        centroid: impl Into<Vec2<f32>>,
    ) -> Triangle {
        let centroid = centroid.into();

        Triangle {
            top: centroid + top.into(),
            left: centroid + left.into(),
            right: centroid + right.into(),
        }
    }
}

impl Rectangle {
    pub fn from(
        width: impl Into<f32>,
        height: impl Into<f32>,
        centroid: impl Into<Vec2<f32>>,
    ) -> Rectangle {
        let width = width.into();
        let height = height.into();
        let centroid = centroid.into();

        Rectangle {
            x: centroid.x - width * 0.5,
            y: centroid.y - height * 0.5,
            width,
            height,
        }
    }
}
