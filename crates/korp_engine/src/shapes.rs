use korp_math::{Flint, Vec2};

#[derive(Copy, Clone, Debug)]
pub struct Line<T> {
    pub start: Vec2<T>,
    pub end: Vec2<T>,
}

#[derive(Copy, Clone, Debug)]
pub struct Triangle<T> {
    pub top: Vec2<T>,
    pub left: Vec2<T>,
    pub right: Vec2<T>,
}

#[derive(Copy, Clone, Debug)]
pub struct Rectangle<T> {
    pub x: T,
    pub y: T,
    pub width: T,
    pub height: T,
}

impl Triangle<f32> {
    pub fn from(
        top: impl Into<Vec2<f32>>,
        left: impl Into<Vec2<f32>>,
        right: impl Into<Vec2<f32>>,
        centroid: impl Into<Vec2<f32>>,
    ) -> Triangle<f32> {
        let centroid = centroid.into();

        Triangle {
            top: centroid + top.into(),
            left: centroid + left.into(),
            right: centroid + right.into(),
        }
    }
}

impl Rectangle<f32> {
    pub fn from(
        width: impl Into<f32>,
        height: impl Into<f32>,
        centroid: impl Into<Vec2<f32>>,
    ) -> Rectangle<f32> {
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

impl Rectangle<Flint> {
    pub fn overlaps(&self, rect: &Rectangle<Flint>) -> bool {
        self.x < rect.x + rect.width
            && self.x + self.width > rect.x
            && self.y < rect.y + rect.height
            && self.y + self.height > rect.y
    }
}

impl Into<Rectangle<f32>> for Rectangle<Flint> {
    fn into(self) -> Rectangle<f32> {
        Rectangle {
            x: self.x.to_f32(),
            y: self.y.to_f32(),
            width: self.width.to_f32(),
            height: self.height.to_f32(),
        }
    }
}
