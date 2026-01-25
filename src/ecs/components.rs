use korp_engine::{color::Color, misc::Morph};
use korp_math::{Flint, Vec2};

use crate::ecs::{entities::Entity, sparse_set::SparseSet};

pub mod traits;

pub struct Components {
    // TODO: add render bodies?
    pub bodies: SparseSet<Morph<Body>>,
    pub motions: SparseSet<Motion>,
}

impl Components {
    pub fn new() -> Self {
        Self {
            bodies: SparseSet::new(u16::MAX as usize),
            motions: SparseSet::new(u16::MAX as usize),
        }
    }

    pub fn destroy(&mut self, entity: Entity) {
        self.bodies.remove(entity);
        self.motions.remove(entity);
    }
}

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

#[derive(Copy, Clone, Debug)]
pub enum Shape {
    Triangle(Triangle),
    Rectangle(Rectangle),
}

#[derive(Copy, Clone, Debug)]
pub struct Triangle {
    pub top: Vec2<Flint>,
    pub left: Vec2<Flint>,
    pub right: Vec2<Flint>,
}

#[derive(Copy, Clone, Debug)]
pub struct Rectangle {
    pub width: Flint,
    pub height: Flint,
}
