use korp_engine::{misc::Morph, shapes::Rectangle as EngineRectangle};
use korp_math::{Flint, Random, Vec2};

use crate::{
    bus::events::CosmosEvent,
    ecs::{
        commands::{Command, SpawnKind},
        components::{Body, Components, Rectangle, Shape, Triangle, traits::Vertexable},
    },
    quadtree::Quadtree,
};

pub const COSMIC_DRAG: Flint = Flint::new(0, Flint::POINT_ONE * 2);

pub struct Executor {}

impl Executor {
    pub fn new() -> Self {
        Self {}
    }

    pub fn execute(
        &mut self,
        cosmos_bounds: EngineRectangle<Flint>,
        random: &mut Random,
        components: &mut Components,
        commands: &mut Vec<Command>,
        quadtree: &mut Quadtree,
        events: &mut Vec<CosmosEvent>,
    ) {
        morph_body(components);
        morph_vertices(components);
        motion(components);
        vertices(components);
        hitbox(components);
        rebuild_quadtree(components, quadtree);
        spawn_protection(components);
        collision(components, quadtree, events);
        out_of_cosmos_bounds(cosmos_bounds, components, commands);
        constant_accelerate(components, commands);
        exhaust_emitter(components, random, commands);
        lifetime(components, commands);

        morph_body_render(components);
        morph_hitbox_render(components);
        body_render(components);
        hitbox_render(components);
        quadtree_nodes_render(components, quadtree);
        cosmos_bounds_render(components, cosmos_bounds);
    }
}

fn morph_body(components: &mut Components) {
    for (_, body) in components.logic.bodies.iter_mut() {
        body.old = body.new;
    }
}

fn morph_vertices(components: &mut Components) {
    for (_, vertices) in components.logic.vertices.iter_mut() {
        std::mem::swap(&mut vertices.old, &mut vertices.new);
    }
}

fn vertices(components: &mut Components) {
    for (entity, body) in components.logic.bodies.iter() {
        if let Some(vertices) = components.logic.vertices.get_mut(entity) {
            vertices.new = body.new.vertices();
        } else {
            let old = body.old.vertices();
            let new = body.new.vertices();

            components
                .logic
                .vertices
                .insert(*entity, Morph::new(old, new));
        }
    }
}

fn hitbox(components: &mut Components) {
    for (&entity, vertices) in components.logic.vertices.iter() {
        let Some((xmin, xmax, ymin, ymax)) = vertices
            .old
            .iter()
            .chain(vertices.new.iter())
            .map(|v| (v.x, v.x, v.y, v.y))
            .reduce(
                |(axmin, axmax, aymin, aymax), (bxmin, bxmax, bymin, bymax)| {
                    (
                        axmin.min(bxmin),
                        axmax.max(bxmax),
                        aymin.min(bymin),
                        aymax.max(bymax),
                    )
                },
            )
        else {
            continue;
        };

        components.logic.hitboxes.insert(
            entity,
            EngineRectangle {
                x: xmin,
                y: ymin,
                width: xmax - xmin,
                height: ymax - ymin,
            },
        );
    }
}

fn spawn_protection(components: &mut Components) {
    let mut removed = Vec::new();

    for (entity, _) in components.logic.spawn_protections.iter() {
        let Some(owner) = components.logic.owners.get(entity) else {
            continue;
        };

        let Some(hitbox_entity) = components.logic.hitboxes.get(entity) else {
            continue;
        };

        let Some(hitbox_owner) = components.logic.hitboxes.get(&owner.entity) else {
            continue;
        };

        if !hitbox_owner.overlaps(hitbox_entity) {
            removed.push(*entity);
        }
    }

    for entity in removed {
        components.logic.spawn_protections.remove(entity);
    }
}

fn collision(components: &mut Components, quadtree: &Quadtree, events: &mut Vec<CosmosEvent>) {
    let project = |vertices: &[Vec2<Flint>], axis: Vec2<Flint>| {
        let mut min = axis.dot(&vertices[0]);
        let mut max = min;

        for i in 1..vertices.len() {
            let projection = axis.dot(&vertices[i]);
            if projection < min {
                min = projection;
            } else if projection > max {
                max = projection;
            }
        }

        (min, max)
    };

    let separated = |v1: &[Vec2<Flint>], v2: &[Vec2<Flint>], axis: &[Vec2<Flint>]| {
        for i in 0..axis.len() {
            let a = axis[i];
            let b = axis[(i + 1) % axis.len()];
            let axis = (b - a).perp();

            let (min1, max1) = project(v1, axis);
            let (min2, max2) = project(v2, axis);

            if max1 < min2 || max2 < min1 {
                return true;
            }
        }

        false
    };

    let intersecting = |v1, v2| {
        if separated(v1, v2, v1) {
            return false;
        }

        if separated(v1, v2, v2) {
            return false;
        }

        true
    };

    for node in quadtree.nodes() {
        let group = node
            .content()
            .iter()
            .filter_map(|(entity, hitbox)| {
                let filter = components.logic.collision_filters.get(entity);
                let vertices = components.logic.vertices.get(entity);

                if let (Some(filter), Some(vertices)) = (filter, vertices) {
                    return Some((entity, hitbox, filter, vertices));
                }

                None
            })
            .collect::<Vec<_>>();

        let mut current = &group[..];

        while let Some(((entity1, hitbox1, filter1, vertices1), remaining)) = current.split_first()
        {
            current = remaining;

            for (entity2, hitbox2, filter2, vertices2) in remaining {
                if !filter1.is_collidable(filter2) {
                    continue;
                }

                if !hitbox1.overlaps(hitbox2) {
                    continue;
                }

                // TODO: need to check values between old and new + rotation
                if intersecting(&vertices1.new, &vertices2.new) {
                    events.push(CosmosEvent::Collided {
                        alpha: **entity1,
                        beta: **entity2,
                        mtv: Flint::ZERO,
                    });
                }
            }
        }
    }
}

fn morph_body_render(components: &mut Components) {
    for (_, body) in components.render.bodies.iter_mut() {
        body.old = body.new;
    }
}

fn morph_hitbox_render(components: &mut Components) {
    for (_, hitbox) in components.render.hitboxes.iter_mut() {
        hitbox.old = hitbox.new;
    }
}

fn body_render(components: &mut Components) {
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
            components.render.bodies.insert(
                entity,
                Morph::new(
                    Body {
                        centroid: lb.old.centroid.into(),
                        rotation: lb.old.rotation.into(),
                        shape: match lb.old.shape {
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
                        color: lb.old.color,
                    },
                    body,
                ),
            );
        }
    }
}

fn hitbox_render(components: &mut Components) {
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

fn quadtree_nodes_render(components: &mut Components, quadtree: &Quadtree) {
    components.render.quadtree_nodes.clear();

    for node in quadtree.nodes() {
        components.render.quadtree_nodes.push(node.bounds().into());
    }
}

fn cosmos_bounds_render(components: &mut Components, cosmos_bounds: EngineRectangle<Flint>) {
    components.render.cosmos_bounds = cosmos_bounds.into();
}

fn motion(components: &mut Components) {
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

fn out_of_cosmos_bounds(
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

fn constant_accelerate(components: &mut Components, commands: &mut Vec<Command>) {
    for (&entity, _) in components.logic.constant_accelerators.iter() {
        commands.push(Command::Accelerate(entity));
    }
}

fn rebuild_quadtree(components: &Components, quadtree: &mut Quadtree) {
    quadtree.clear();

    for (entity, hitbox) in components.logic.hitboxes.iter() {
        quadtree.insert(*entity, *hitbox);
    }
}

fn exhaust_emitter(components: &mut Components, random: &mut Random, commands: &mut Vec<Command>) {
    for (entity, emitter) in components.logic.exhaust_emitters.iter_mut() {
        if emitter.lifetime == 0 {
            continue;
        }

        emitter.lifetime -= 1;

        if let Some(body) = components.logic.bodies.get(entity) {
            let direction = body.new.rotation * -1;

            let half = emitter.size / 2;
            for i in 0..emitter.size {
                let centroid = body.new.centroid
                    + match body.new.shape {
                        Shape::Triangle(triangle) => {
                            let middle = (triangle.left + triangle.right) * Flint::ZERO_FIVE;
                            Vec2::new(middle.x, middle.y + Flint::new(i - half, 0))
                        }
                        Shape::Rectangle(rectangle) => {
                            Vec2::new(rectangle.width * Flint::ZERO_FIVE, rectangle.height)
                        }
                    }
                    .rotated_v(body.new.rotation);

                commands.push(Command::Spawn {
                    id: None,
                    kind: SpawnKind::Particle {
                        centroid,
                        direction,
                        speed: Flint::new(random.range(0, 4) as i16, random.range(0, 256) as u16),
                        lifetime: random.range(4, 20) as u32,
                    },
                });
            }
        }
    }
}

fn lifetime(components: &mut Components, commands: &mut Vec<Command>) {
    for (entity, lifetime) in components.logic.lifetimes.iter_mut() {
        if lifetime.value > 0 {
            lifetime.value -= 1;
        } else {
            commands.push(Command::Kill(*entity));
        }
    }
}
