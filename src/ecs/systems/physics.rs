use korp_engine::{misc::Morph, shapes::Rectangle as EngineRectangle};
use korp_math::{Flint, Vec2};

use crate::{
    bus::events::CosmosEvent,
    ecs::{
        commands::Command,
        components::{Body, Components, Rectangle, Shape, Triangle, traits::Hitboxable},
    },
    quadtree::Quadtree,
};

pub const COSMIC_DRAG: Flint = Flint::new(0, Flint::POINT_ONE * 2);

pub fn morph_body(components: &mut Components) {
    for (_, body) in components.logic.bodies.iter_mut() {
        body.old = body.new;
    }
}

pub fn hitbox(components: &mut Components) {
    for (&entity, body) in components.logic.bodies.iter() {
        components.logic.hitboxes.insert(entity, body.hitbox());
    }
}

pub fn collision(components: &mut Components, quadtree: &Quadtree, events: &mut Vec<CosmosEvent>) {
    for node in quadtree.nodes() {
        let mut group = node
            .content()
            .iter()
            .filter_map(|(entity, hitbox)| {
                if let Some(filter) = components.logic.collision_filters.get(entity) {
                    return Some((entity, hitbox, filter));
                }

                None
            })
            .peekable();

        while let Some((entity1, hitbox1, filter1)) = group.next() {
            let Some((entity2, hitbox2, filter2)) = group.peek() else {
                break;
            };

            if !filter1.is_collidable(filter2) {
                continue;
            }

            if hitbox1.overlaps(hitbox2) {
                // TODO: use SAT
                events.push(CosmosEvent::Collided {
                    alpha: *entity1,
                    beta: **entity2,
                    mtv: Flint::ZERO,
                });
            }
        }
    }
}

pub fn morph_body_render(components: &mut Components) {
    for (_, body) in components.render.bodies.iter_mut() {
        body.old = body.new;
    }
}

pub fn morph_hitbox_render(components: &mut Components) {
    for (_, hitbox) in components.render.hitboxes.iter_mut() {
        hitbox.old = hitbox.new;
    }
}

pub fn body_render(components: &mut Components) {
    for (&entity, lb) in components.logic.bodies.iter() {
        let body = Body {
            centroid: lb.new.centroid.into(),
            rotation: lb.new.rotation.into(),
            shape: match lb.new.shape {
                Shape::Triangle(triangle) => Shape::Triangle(Triangle {
                    top: triangle.top.into(),
                    left: triangle.left.into(),
                    right: triangle.right.into(),
                }),
                Shape::Rectangle(rectangle) => Shape::Rectangle(Rectangle {
                    width: rectangle.width.into(),
                    height: rectangle.height.into(),
                }),
            },
            color: lb.new.color,
        };

        if let Some(rb) = components.render.bodies.get_mut(&entity) {
            rb.new = body;
        } else {
            components.render.bodies.insert(entity, Morph::one(body));
        }
    }
}

pub fn hitbox_render(components: &mut Components) {
    for (&entity, lhb) in components.logic.hitboxes.iter() {
        let rectangle = EngineRectangle {
            x: lhb.x.into(),
            y: lhb.y.into(),
            width: lhb.width.into(),
            height: lhb.height.into(),
        };

        if let Some(rhb) = components.render.hitboxes.get_mut(&entity) {
            rhb.new = rectangle;
        } else {
            components
                .render
                .hitboxes
                .insert(entity, Morph::one(rectangle));
        }
    }
}

pub fn quadtree_nodes_render(components: &mut Components, quadtree: &Quadtree) {
    components.render.quadtree_nodes.clear();

    for node in quadtree.nodes() {
        components.render.quadtree_nodes.push(node.bounds().into());
    }
}

pub fn cosmos_bounds_render(components: &mut Components, cosmos_bounds: EngineRectangle<Flint>) {
    components.render.cosmos_bounds = cosmos_bounds.into();
}

pub fn motion(components: &mut Components) {
    for (entity, motion) in components.logic.motions.iter_mut() {
        let Some(body) = components.logic.bodies.get_mut(entity) else {
            continue;
        };

        // apply cosmic drag for rotation
        if motion.rotation_speed < Flint::ZERO {
            motion.rotation_speed += COSMIC_DRAG;

            if motion.rotation_speed > Flint::ZERO {
                motion.rotation_speed = Flint::ZERO;
            }
        } else if motion.rotation_speed > Flint::ZERO {
            motion.rotation_speed -= COSMIC_DRAG;

            if motion.rotation_speed < Flint::ZERO {
                motion.rotation_speed = Flint::ZERO;
            }
        }

        // ensure min/max rotation speed
        if motion.rotation_speed > motion.rotation_speed_maximum {
            motion.rotation_speed = motion.rotation_speed_maximum;
        } else if motion.rotation_speed < motion.rotation_speed_minimum {
            motion.rotation_speed = motion.rotation_speed_minimum;
        }

        // set updated rotation
        if motion.rotation_speed != Flint::ZERO {
            body.new.rotation = body.new.rotation.rotated(motion.rotation_speed);
        }

        let direction = motion.velocity.normalized();

        // apply cosmic drag for velocity
        motion.velocity -= direction * COSMIC_DRAG;

        // make a full stop if entity has suddenly switched direction
        if direction.dot(&motion.velocity.normalized()) < Flint::ZERO {
            motion.velocity = Vec2::ZERO;
        }

        let speed = motion.velocity.len_sqr();
        let speed_max = motion.speed_maximum * motion.speed_maximum;
        let speed_min = -(motion.speed_minimum * motion.speed_minimum);

        // ensure min/max velocity
        if speed > speed_max {
            motion.velocity = direction * motion.speed_maximum;
        } else if speed < speed_min {
            motion.velocity = direction * motion.speed_minimum;
        }

        body.new.centroid += motion.velocity;
    }
}

pub fn out_of_cosmos_bounds(
    bounds: EngineRectangle<Flint>,
    components: &mut Components,
    commands: &mut Vec<Command>,
) {
    for (&entity, hitbox) in components.logic.hitboxes.iter() {
        if !bounds.overlaps(hitbox) {
            commands.push(Command::Kill(entity));
        }
    }
}

pub fn constant_accelerate(components: &mut Components, commands: &mut Vec<Command>) {
    for (&entity, _) in components.logic.constant_accelerators.iter() {
        commands.push(Command::Accelerate(entity));
    }
}

pub fn rebuild_quadtree(components: &Components, quadtree: &mut Quadtree) {
    quadtree.clear();

    for (entity, hitbox) in components.logic.hitboxes.iter() {
        quadtree.insert(*entity, *hitbox);
    }
}
