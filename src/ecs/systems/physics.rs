use korp_math::{Flint, Vec2};

use crate::ecs::cosmos::Components;

pub const COSMIC_DRAG: Flint = Flint::new(0, Flint::POINT_ONE * 2);

pub fn morph_body(components: &mut Components) {
    for (_, body) in components.bodies.iter_mut() {
        body.old = body.new;
    }
}

pub fn motion(components: &mut Components) {
    for (entity, motion) in components.motions.iter_mut() {
        let Some(body) = components.bodies.get_mut(entity) else {
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
