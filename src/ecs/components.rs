use korp_engine::{color::Color, misc::Morph, shapes::Rectangle as EngineRectangle};
use korp_math::{Flint, Vec2};

use crate::ecs::{entities::Entity, sparse_set::SparseSet};

pub mod collision_filter;
pub mod traits;

pub struct Logic {
    pub bodies: SparseSet<Morph<Body<Flint>>>,
    pub hitboxes: SparseSet<EngineRectangle<Flint>>,
    pub motions: SparseSet<Motion>,
    pub constant_accelerators: SparseSet<ConstantAccelerator>,
    pub collision_filters: SparseSet<CollisionFilter>,
    pub vertices: SparseSet<Morph<Vec<Vec2<Flint>>>>,
    pub owners: SparseSet<Owner>,
    pub spawn_protections: SparseSet<SpawnProtection>,
    pub exhaust_emitters: SparseSet<ExhaustEmitter>,
    pub particles: Vec<Particle>,
}

pub struct Render {
    pub bodies: SparseSet<Morph<Body<f32>>>,
    pub hitboxes: SparseSet<Morph<EngineRectangle<f32>>>,
    pub cosmos_bounds: EngineRectangle<f32>,
    pub quadtree_nodes: Vec<EngineRectangle<f32>>,
    pub particles: Vec<Morph<Body<f32>>>,
}

pub struct Components {
    pub logic: Logic,
    pub render: Render,
}

impl Components {
    pub fn new(cosmos_bounds: EngineRectangle<Flint>) -> Self {
        let capacity = u16::MAX as usize;

        Self {
            logic: Logic {
                bodies: SparseSet::new(capacity),
                hitboxes: SparseSet::new(capacity),
                motions: SparseSet::new(capacity),
                constant_accelerators: SparseSet::new(capacity),
                collision_filters: SparseSet::new(capacity),
                vertices: SparseSet::new(capacity),
                owners: SparseSet::new(capacity),
                spawn_protections: SparseSet::new(capacity),
                exhaust_emitters: SparseSet::new(capacity),
                particles: Vec::new(),
            },
            render: Render {
                bodies: SparseSet::new(capacity),
                hitboxes: SparseSet::new(capacity),
                cosmos_bounds: cosmos_bounds.into(),
                quadtree_nodes: Vec::new(),
                particles: Vec::new(),
            },
        }
    }

    pub fn destroy(&mut self, entity: Entity) {
        self.logic.bodies.remove(entity);
        self.logic.hitboxes.remove(entity);
        self.logic.motions.remove(entity);
        self.logic.constant_accelerators.remove(entity);
        self.logic.collision_filters.remove(entity);
        self.logic.vertices.remove(entity);
        self.logic.owners.remove(entity);
        self.logic.spawn_protections.remove(entity);
        self.logic.exhaust_emitters.remove(entity);

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

pub struct ConstantAccelerator;

pub struct CollisionFilter {
    pub category: u32,
    pub mask: u32,
}

pub struct Owner {
    pub entity: Entity,
}

pub struct SpawnProtection;

pub struct ExhaustEmitter {
    pub lifetime_maximum: u32,
    pub lifetime: u32,
    pub width: i16,
    pub relative_position: Vec2<Flint>,
    pub relative_direction: Vec2<Flint>,
}

pub struct Particle {
    pub lifetime: u32,
    pub velocity: Vec2<Flint>,
    pub body: Morph<Body<Flint>>,
}
