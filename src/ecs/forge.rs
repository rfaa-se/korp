use korp_engine::{color::Color, misc::Morph};
use korp_math::{Flint, Vec2};

use crate::ecs::{
    components::{Body, Brain, Components, Motion, Rectangle, Shape, Triangle},
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

        let body = Body {
            centroid,
            rotation: Vec2::new(Flint::ZERO, Flint::NEG_ONE),
            shape: Shape::Triangle(Triangle {
                top: Vec2::new(Flint::new(50, 0), Flint::new(0, 0)),
                left: Vec2::new(Flint::new(-25, 0), Flint::new(-30, 0)),
                right: Vec2::new(Flint::new(-25, 0), Flint::new(30, 0)),
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

        entity
    }

    pub fn rectangle(&mut self, centroid: Vec2<Flint>, components: &mut Components) -> Entity {
        let entity = self.factory.create();

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

        entity
    }

    pub fn projectile(
        &mut self,
        centroid: Vec2<Flint>,
        rotation: Vec2<Flint>,
        components: &mut Components,
    ) -> Entity {
        let entity = self.factory.create();

        let body = Body {
            centroid,
            rotation,
            shape: Shape::Rectangle(Rectangle {
                width: Flint::new(16, 0),
                height: Flint::new(10, 0),
            }),
            color: Color::GREEN,
        };

        components.logic.bodies.insert(entity, Morph::one(body));

        components.logic.motions.insert(
            entity,
            Motion {
                velocity: rotation + rotation * Flint::new(10, 0),
                speed_maximum: Flint::new(15, 0),
                speed_minimum: Flint::ZERO,
                acceleration: COSMIC_DRAG,
                rotation_speed: Flint::ZERO,
                rotation_speed_maximum: Flint::ZERO,
                rotation_speed_minimum: Flint::ZERO,
                rotation_acceleration: Flint::ZERO,
            },
        );

        components.logic.brains.projectiles.insert(entity, Brain);

        entity
    }
}
