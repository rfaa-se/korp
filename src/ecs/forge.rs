use korp_engine::{color::Color, misc::Morph};
use korp_math::{Flint, Random, Vec2};

use crate::ecs::{
    components::{
        Accelerator, Body, CollisionFilter, Components, DeathExplosive, ExhaustEmitter,
        LeftRotator, Motion, Owner, Particle, Rectangle, RightRotator, Shape, SpawnProtection,
        Triangle,
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
                width: Flint::from_i16(7),
                relative_position: Vec2::new(
                    (left.x + right.x) * Flint::ZERO_FIVE,
                    (left.y + right.y) * Flint::ZERO_FIVE,
                ),
                relative_direction: Vec2::new(Flint::NEG_ONE, Flint::ZERO),
            },
        );

        components
            .logic
            .death_explosives
            .insert(entity, DeathExplosive);

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

        components.logic.accelerators.insert(entity, Accelerator);

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

    pub fn explode(
        &mut self,
        entity: Entity,
        random: &mut Random,
        components: &mut Components,
        graveyard: &Components,
    ) {
        let Some(body) = graveyard.logic.bodies.get(&entity) else {
            return;
        };

        let rnd_point = |t: Triangle<Flint>, rnd: &mut Random| {
            let r1 = Flint::new(0, rnd.range_u16(0, u16::MAX)).sqrt();
            let r2 = Flint::new(0, rnd.range_u16(0, u16::MAX));
            let one = Flint::ONE;

            let x = (one - r1) * t.top.x + (r1 * (one - r2)) * t.left.x + (r1 * r2) * t.right.x;
            let y = (one - r1) * t.top.y + (r1 * (one - r2)) * t.left.y + (r1 * r2) * t.right.y;

            Vec2::new(x, y)
        };

        let calc_centroid = |a: Vec2<Flint>, b: Vec2<Flint>, c: Vec2<Flint>| {
            Vec2::new((a.x + b.x + c.x) / 3.into(), (a.y + b.y + c.y) / 3.into())
        };

        let rotation = body.new.rotation;
        let centroid_origin = body.new.centroid;

        let calc_centroid_shard =
            |a: Vec2<Flint>, b: Vec2<Flint>, c: Vec2<Flint>, centroid: Vec2<Flint>| {
                let centroid_local = calc_centroid(a, b, c);
                let shard = Triangle {
                    top: a - centroid_local,
                    left: b - centroid_local,
                    right: c - centroid_local,
                };

                (centroid + centroid_local.rotated_v(rotation), shard)
            };

        match body.new.shape {
            Shape::Triangle(triangle) => {
                let size = 3;
                let mut shards = vec![(body.new.centroid, triangle)];

                while shards.len() < size {
                    let idx = random.range_usize(0, shards.len());
                    let (centroid, shard) = shards.remove(idx);
                    let half = Triangle {
                        top: shard.top * Flint::ZERO_FIVE,
                        left: shard.left * Flint::ZERO_FIVE,
                        right: shard.right * Flint::ZERO_FIVE,
                    };

                    let p = rnd_point(half, random);

                    let (c1, s1) = calc_centroid_shard(shard.top, shard.left, p, centroid);
                    let (c2, s2) = calc_centroid_shard(shard.left, shard.right, p, centroid);
                    let (c3, s3) = calc_centroid_shard(shard.right, shard.top, p, centroid);

                    shards.push((c1, s1));
                    shards.push((c2, s2));
                    shards.push((c3, s3));
                }

                for (i, (centroid, shard)) in shards.into_iter().enumerate() {
                    let e = self.factory.create();
                    let body = Body {
                        centroid,
                        rotation,
                        shape: Shape::Triangle(shard),
                        color: body.new.color,
                    };

                    if i % 2 == 0 {
                        components.logic.left_rotators.insert(e, LeftRotator);
                    } else {
                        components.logic.right_rotators.insert(e, RightRotator);
                    }

                    components.logic.bodies.insert(e, Morph::one(body));
                    components.logic.motions.insert(
                        e,
                        Motion {
                            velocity: (centroid - centroid_origin).normalized() * Flint::new(8, 0),
                            speed_maximum: 8.into(),
                            speed_minimum: 0.into(),
                            acceleration: Flint::new(1, 0),
                            rotation_speed: 10.into(),
                            rotation_speed_maximum: 10.into(),
                            rotation_speed_minimum: 0.into(),
                            rotation_acceleration: Flint::new(0, Flint::POINT_ONE),
                        },
                    );
                }
            }
            Shape::Rectangle(_) => {
                // TODO
            }
        }
    }
}
