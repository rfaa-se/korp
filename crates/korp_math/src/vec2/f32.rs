use std::ops::{Add, AddAssign, Mul, Sub};

use crate::{Flint, Vec2};

impl Vec2<f32> {
    pub fn perp(&self) -> Self {
        Self {
            x: -self.y,
            y: self.x,
        }
    }

    pub fn normalized(&self) -> Self {
        let len = self.len();

        if len == 0.0 {
            return Self { x: 0.0, y: 0.0 };
        }

        Self {
            x: self.x / len,
            y: self.y / len,
        }
    }

    pub fn len(&self) -> f32 {
        self.dot(self).sqrt()
    }

    pub fn dot(&self, v: &Vec2<f32>) -> f32 {
        self.x * v.x + self.y * v.y
    }

    pub fn angle(&self) -> f32 {
        self.y.atan2(self.x)
    }

    pub fn from_angle(radians: f32) -> Self {
        Self {
            x: radians.cos(),
            y: radians.sin(),
        }
    }
}

impl From<Vec2<Flint>> for Vec2<f32> {
    fn from(value: Vec2<Flint>) -> Self {
        Vec2::new(value.x.into(), value.y.into())
    }
}

impl Add for Vec2<f32> {
    type Output = Vec2<f32>;

    fn add(self, rhs: Self) -> Self::Output {
        Vec2::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl AddAssign for Vec2<f32> {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for Vec2<f32> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec2::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Mul<f32> for Vec2<f32> {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Vec2::new(self.x * rhs, self.y * rhs)
    }
}
