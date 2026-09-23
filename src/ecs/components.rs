use korp_engine::{color::Color, misc::Morph, shapes::Rectangle as EngineRectangle};
use korp_math::{Flint, Vec2};

use crate::ecs::{entities::Entity, sparse_set::SparseSet};

pub mod collision_filter;
pub mod traits;

pub struct Logic {
    pub particles: Vec<Particle>,
    pub bodies: SparseSet<Morph<Body<Flint>>>,
    pub hitboxes: SparseSet<EngineRectangle<Flint>>,
    pub motions: SparseSet<Motion>,
    pub accelerators: SparseSet<Accelerator>,
    pub left_rotators: SparseSet<LeftRotator>,
    pub right_rotators: SparseSet<RightRotator>,
    pub collision_filters: SparseSet<CollisionFilter>,
    pub vertices: SparseSet<Morph<Vec<Vec2<Flint>>>>,
    pub owners: SparseSet<Owner>,
    pub spawn_protections: SparseSet<SpawnProtection>,
    pub exhaust_emitters: SparseSet<ExhaustEmitter>,
    pub death_explosives: SparseSet<DeathExplosive>,
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

impl Logic {
    fn new() -> Self {
        Self {
            particles: Vec::new(),
            bodies: SparseSet::new(),
            hitboxes: SparseSet::new(),
            motions: SparseSet::new(),
            accelerators: SparseSet::new(),
            left_rotators: SparseSet::new(),
            right_rotators: SparseSet::new(),
            collision_filters: SparseSet::new(),
            vertices: SparseSet::new(),
            owners: SparseSet::new(),
            spawn_protections: SparseSet::new(),
            exhaust_emitters: SparseSet::new(),
            death_explosives: SparseSet::new(),
        }
    }
}

impl Render {
    fn new(cosmos_bounds: EngineRectangle<Flint>) -> Self {
        Self {
            bodies: SparseSet::new(),
            hitboxes: SparseSet::new(),
            cosmos_bounds: cosmos_bounds.into(),
            quadtree_nodes: Vec::new(),
            particles: Vec::new(),
        }
    }
}

impl Components {
    pub fn new(cosmos_bounds: EngineRectangle<Flint>) -> Self {
        Self {
            logic: Logic::new(),
            render: Render::new(cosmos_bounds),
        }
    }

    pub fn destroy(&mut self, entity: Entity) {
        let l = &mut self.logic;
        let r = &mut self.render;

        l.bodies.remove(entity);
        l.hitboxes.remove(entity);
        l.motions.remove(entity);
        l.accelerators.remove(entity);
        l.left_rotators.remove(entity);
        l.right_rotators.remove(entity);
        l.collision_filters.remove(entity);
        l.vertices.remove(entity);
        l.owners.remove(entity);
        l.spawn_protections.remove(entity);
        l.exhaust_emitters.remove(entity);
        l.death_explosives.remove(entity);

        r.bodies.remove(entity);
        r.hitboxes.remove(entity);
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

#[derive(Copy, Clone, Debug)]
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

pub struct Accelerator;
pub struct LeftRotator;
pub struct RightRotator;

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
    pub width: Flint,
    pub relative_position: Vec2<Flint>,
    pub relative_direction: Vec2<Flint>,
}

pub struct Particle {
    pub lifetime: u32,
    pub velocity: Vec2<Flint>,
    pub body: Morph<Body<Flint>>,
}

pub struct DeathExplosive;
