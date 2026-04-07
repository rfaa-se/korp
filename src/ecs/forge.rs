use korp_engine::{color::Color, misc::Morph};
use korp_math::{Flint, Vec2};

use crate::ecs::{
    components::{
        Body, CollisionFilter, Components, ConstantAccelerator, ExhaustEmitter, Motion, Owner,
        Particle, Rectangle, Shape, SpawnProtection, Triangle,
    },
    entities::{Entity, EntityFactory},
    systems::COSMIC_DRAG,
};

pub struct Forge {
    factory: EntityFactory,
}

impl Forge {
    pub fn new() -> Self {
        Self {
            factory: EntityFactory::new(),
        }
    }

    pub fn destroy(&mut self, entity: Entity, components: &mut Components) {
        self.factory.destroy(entity);
        components.destroy(entity);
    }

    pub fn triangle(&mut self, centroid: Vec2<Flint>, components: &mut Components) -> Entity {
        let entity = self.factory.create();

        // left
        // |\
        // | \
        // |  > top
        // | /
        // |/
        // right
        //
        let top = Vec2::new(Flint::new(50, 0), Flint::new(0, 0));
        let left = Vec2::new(Flint::new(-25, 0), Flint::new(-30, 0));
        let right = Vec2::new(Flint::new(-25, 0), Flint::new(30, 0));
        let body = Body {
            centroid,
            rotation: Vec2::new(Flint::ZERO, Flint::NEG_ONE),
            shape: Shape::Triangle(Triangle { top, left, right }),
            color: Color::GREEN,
        };

        components.logic.bodies.insert(entity, Morph::one(body));

        components.logic.motions.insert(
            entity,
            Motion {
                velocity: Vec2::ZERO,
                speed_maximum: Flint::new(15, 0),
                speed_minimum: -Flint::new(10, 0),
                acceleration: Flint::new(1, Flint::POINT_ONE * 3),
                rotation_speed: Flint::ZERO,
                rotation_speed_maximum: Flint::new(16, 0),
                rotation_speed_minimum: -Flint::new(16, 0),
                rotation_acceleration: Flint::new(1, 0),
            },
        );

        components.logic.collision_filters.insert(
            entity,
            CollisionFilter {
                category: CollisionFilter::TRIANGLE,
                mask: CollisionFilter::PROJECTILE
                    | CollisionFilter::TRIANGLE
                    | CollisionFilter::RECTANGLE,
            },
        );

        components.logic.exhaust_emitters.insert(
            entity,
            ExhaustEmitter {
                lifetime_maximum: 5,
                lifetime: 0,
                width: 7,
                relative_position: Vec2::new(
                    (left.x + right.x) * Flint::ZERO_FIVE,
                    (left.y + right.y) * Flint::ZERO_FIVE,
                ),
                relative_direction: Vec2::new(Flint::NEG_ONE, Flint::ZERO),
            },
        );

        entity
    }

    pub fn rectangle(&mut self, centroid: Vec2<Flint>, components: &mut Components) -> Entity {
        let entity = self.factory.create();

        //  height
        // ________
        // |       | width
        // |_______|
        //
        let body = Body {
            centroid,
            rotation: Vec2::new(Flint::ZERO, Flint::NEG_ONE),
            shape: Shape::Rectangle(Rectangle {
                width: Flint::new(40, 0),
                height: Flint::new(60, 0),
            }),
            color: Color::GREEN,
        };

        components.logic.bodies.insert(entity, Morph::one(body));

        components.logic.motions.insert(
            entity,
            Motion {
                velocity: Vec2::ZERO,
                speed_maximum: Flint::new(15, 0),
                speed_minimum: -Flint::new(10, 0),
                acceleration: Flint::new(1, Flint::POINT_ONE * 3),
                rotation_speed: Flint::ZERO,
                rotation_speed_maximum: Flint::new(16, 0),
                rotation_speed_minimum: -Flint::new(16, 0),
                rotation_acceleration: Flint::new(1, 0),
            },
        );

        components.logic.collision_filters.insert(
            entity,
            CollisionFilter {
                category: CollisionFilter::RECTANGLE,
                mask: CollisionFilter::PROJECTILE
                    | CollisionFilter::TRIANGLE
                    | CollisionFilter::RECTANGLE,
            },
        );

        entity
    }

    pub fn projectile(
        &mut self,
        owner: Entity,
        relative_speed: Flint,
        centroid: Vec2<Flint>,
        rotation: Vec2<Flint>,
        components: &mut Components,
    ) -> Entity {
        let entity = self.factory.create();

        let body = Body {
            centroid,
            rotation,
            shape: Shape::Rectangle(Rectangle {
                width: Flint::new(6, 0),
                height: Flint::new(3, 0),
            }),
            color: Color::GREEN,
        };

        components.logic.bodies.insert(entity, Morph::one(body));

        let velocity = rotation * Flint::new(16, 0) + rotation * relative_speed;

        components.logic.motions.insert(
            entity,
            Motion {
                velocity,
                speed_maximum: Flint::new(100, 0),
                speed_minimum: Flint::ZERO,
                acceleration: COSMIC_DRAG,
                rotation_speed: Flint::ZERO,
                rotation_speed_maximum: Flint::ZERO,
                rotation_speed_minimum: Flint::ZERO,
                rotation_acceleration: Flint::ZERO,
            },
        );

        components
            .logic
            .constant_accelerators
            .insert(entity, ConstantAccelerator);

        components.logic.collision_filters.insert(
            entity,
            CollisionFilter {
                category: CollisionFilter::PROJECTILE,
                mask: CollisionFilter::TRIANGLE | CollisionFilter::RECTANGLE,
            },
        );

        components
            .logic
            .owners
            .insert(entity, Owner { entity: owner });

        components
            .logic
            .spawn_protections
            .insert(entity, SpawnProtection);

        entity
    }

    pub fn particle(
        &mut self,
        centroid: Vec2<Flint>,
        direction: Vec2<Flint>,
        speed: Flint,
        lifetime: u32,
        components: &mut Components,
    ) {
        components.logic.particles.push(Particle {
            lifetime,
            velocity: direction * speed,
            body: Morph::one(Body {
                centroid,
                rotation: direction,
                shape: Shape::Rectangle(Rectangle {
                    width: Flint::ONE,
                    height: Flint::ONE,
                }),
                color: Color::BLUE,
            }),
        });
    }
}
