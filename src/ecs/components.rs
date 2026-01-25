use korp_engine::{color::Color, misc::Morph, shapes::Rectangle as ERectangle};
use korp_math::{Flint, Vec2};

use crate::ecs::{entities::Entity, sparse_set::SparseSet};

pub mod traits;

pub struct Logic {
    pub bodies: SparseSet<Morph<Body>>,
    pub hitboxes: SparseSet<ERectangle<Flint>>,
    pub motions: SparseSet<Motion>,
}

pub struct Render {
    pub hitboxes: SparseSet<Morph<ERectangle<f32>>>,
}

pub struct Components {
    pub logic: Logic,
    pub render: Render,
}

impl Components {
    pub fn new() -> Self {
        Self {
            logic: Logic {
                bodies: SparseSet::new(u16::MAX as usize),
                hitboxes: SparseSet::new(u16::MAX as usize),
                motions: SparseSet::new(u16::MAX as usize),
            },
            render: Render {
                hitboxes: SparseSet::new(u16::MAX as usize),
            },
        }
    }

    pub fn destroy(&mut self, entity: Entity) {
        self.logic.bodies.remove(entity);
        self.logic.hitboxes.remove(entity);
        self.logic.motions.remove(entity);

        self.render.hitboxes.remove(entity);
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
