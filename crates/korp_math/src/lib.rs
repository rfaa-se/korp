mod flint;
mod vec2;

use std::f32::consts::{PI, TAU};

pub use flint::*;
pub use vec2::*;

pub fn lerp_angle(a: f32, b: f32, t: f32) -> f32 {
    let mut delta = b - a;

    while delta > PI {
        delta -= TAU;
    }

    while delta < -PI {
        delta += TAU;
    }

    a + delta * t
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
