use std::ops::{Add, AddAssign, Mul, Sub, SubAssign};

use crate::Flint;

#[derive(Copy, Clone)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

impl<T> From<Vec2<T>> for [T; 2] {
    fn from(value: Vec2<T>) -> Self {
        [value.x, value.y]
    }
}

impl<T> Vec2<T> {
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

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
}

impl Vec2<Flint> {
    pub const ZERO: Vec2<Flint> = Vec2::new(Flint::ZERO, Flint::ZERO);

    pub fn perp(&self) -> Self {
        Self {
            x: -self.y,
            y: self.x,
        }
    }

    pub fn normalized(&self) -> Self {
        let len = self.len();

        if len == Flint::ZERO {
            return Self {
                x: Flint::ZERO,
                y: Flint::ZERO,
            };
        }

        Self {
            x: self.x / len,
            y: self.y / len,
        }
    }

    pub fn len(&self) -> Flint {
        self.len_sqr().sqrt()
    }

    pub fn len_sqr(&self) -> Flint {
        self.dot(&self)
    }

    pub fn dot(&self, v: &Vec2<Flint>) -> Flint {
        self.x * v.x + self.y * v.y
    }

    pub fn rotated(&self, degrees: Flint) -> Self {
        let radians = degrees.to_radians();
        let (sin, cos) = radians.sin_cos();

        let x = self.x * cos - self.y * sin;
        let y = self.x * sin + self.y * cos;

        Self { x, y }
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

impl Mul<Flint> for Vec2<Flint> {
    type Output = Self;

    fn mul(self, rhs: Flint) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Mul<i16> for Vec2<Flint> {
    type Output = Self;

    fn mul(self, rhs: i16) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl AddAssign for Vec2<Flint> {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl SubAssign for Vec2<Flint> {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}
