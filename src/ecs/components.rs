use korp_engine::{color::Color, misc::Morph, shapes::Rectangle as EngineRectangle};
use korp_math::{Flint, Vec2};

use crate::ecs::{entities::Entity, sparse_set::SparseSet};

pub mod collision_filter;
pub mod traits;

pub struct Logic {
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
    pub particles: Vec<Particle>,
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

impl Components {
    pub fn new(cosmos_bounds: EngineRectangle<Flint>) -> Self {
        let capacity = u16::MAX as usize;

        Self {
            logic: Logic {
                bodies: SparseSet::new(capacity),
                hitboxes: SparseSet::new(capacity),
                motions: SparseSet::new(capacity),
                accelerators: SparseSet::new(capacity),
                left_rotators: SparseSet::new(capacity),
                right_rotators: SparseSet::new(capacity),
                collision_filters: SparseSet::new(capacity),
                vertices: SparseSet::new(capacity),
                owners: SparseSet::new(capacity),
                spawn_protections: SparseSet::new(capacity),
                exhaust_emitters: SparseSet::new(capacity),
                particles: Vec::new(),
                death_explosives: SparseSet::new(capacity),
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
        self.logic.accelerators.remove(entity);
        self.logic.left_rotators.remove(entity);
        self.logic.right_rotators.remove(entity);
        self.logic.collision_filters.remove(entity);
        self.logic.vertices.remove(entity);
        self.logic.owners.remove(entity);
        self.logic.spawn_protections.remove(entity);
        self.logic.exhaust_emitters.remove(entity);
        self.logic.death_explosives.remove(entity);

        self.render.bodies.remove(entity);
        self.render.hitboxes.remove(entity);
    }

    pub fn transfer(&mut self, entity: Entity, components: &mut Components) {
        let l = &mut self.logic;
        let cl = &mut components.logic;
        let r = &mut self.render;
        let cr = &mut components.render;
        let e = entity;

        transfer(e, &mut l.bodies, &mut cl.bodies);
        transfer(e, &mut l.hitboxes, &mut cl.hitboxes);
        transfer(e, &mut l.motions, &mut cl.motions);
        transfer(e, &mut l.accelerators, &mut cl.accelerators);
        transfer(e, &mut l.left_rotators, &mut cl.left_rotators);
        transfer(e, &mut l.right_rotators, &mut cl.right_rotators);
        transfer(e, &mut l.collision_filters, &mut cl.collision_filters);
        transfer(e, &mut l.vertices, &mut cl.vertices);
        transfer(e, &mut l.owners, &mut cl.owners);
        transfer(e, &mut l.spawn_protections, &mut cl.spawn_protections);
        transfer(e, &mut l.exhaust_emitters, &mut cl.exhaust_emitters);
        transfer(e, &mut l.death_explosives, &mut cl.death_explosives);

        transfer(e, &mut r.bodies, &mut cr.bodies);
        transfer(e, &mut r.hitboxes, &mut cr.hitboxes);
    }
}

fn transfer<T>(entity: Entity, from: &mut SparseSet<T>, to: &mut SparseSet<T>) {
    if let Some(component) = from.remove(entity) {
        to.insert(entity, component);
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
