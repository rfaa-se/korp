use korp_engine::{color::Color, misc::Morph, shapes::Rectangle as EngineRectangle};
use korp_math::{Flint, Vec2};

use crate::ecs::{entities::Entity, sparse_set::SparseSet};

pub mod traits;

pub struct Logic {
    pub bodies: SparseSet<Morph<Body<Flint>>>,
    pub hitboxes: SparseSet<EngineRectangle<Flint>>,
    pub motions: SparseSet<Motion>,
}

pub struct Render {
    pub bodies: SparseSet<Morph<Body<f32>>>,
    pub hitboxes: SparseSet<Morph<EngineRectangle<f32>>>,
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
                bodies: SparseSet::new(u16::MAX as usize),
                hitboxes: SparseSet::new(u16::MAX as usize),
            },
        }
    }

    pub fn destroy(&mut self, entity: Entity) {
        self.logic.bodies.remove(entity);
        self.logic.hitboxes.remove(entity);
        self.logic.motions.remove(entity);

        self.render.bodies.remove(entity);
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
pub struct Body<T> {
    pub centroid: Vec2<T>,
    pub rotation: Vec2<T>,
    pub shape: Shape<T>,
    pub color: Color,
}

#[derive(Copy, Clone, Debug)]
pub enum Shape<T> {
    Triangle(Triangle<T>),
    Rectangle(Rectangle<T>),
}

#[derive(Copy, Clone, Debug)]
pub struct Triangle<T> {
    pub top: Vec2<T>,
    pub left: Vec2<T>,
    pub right: Vec2<T>,
}

#[derive(Copy, Clone, Debug)]
pub struct Rectangle<T> {
    pub width: T,
    pub height: T,
}
