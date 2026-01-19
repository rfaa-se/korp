use std::ops::{Add, AddAssign, Mul, Sub, SubAssign};

use crate::{Flint, Vec2};

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

    pub fn rotated_v(&self, theta: Vec2<Flint>) -> Self {
        Self {
            x: self.x * theta.x - self.y * theta.y,
            y: self.x * theta.y + self.y * theta.x,
        }
    }
}

impl Add for Vec2<Flint> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Vec2<Flint> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
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
