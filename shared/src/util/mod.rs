use std::f32::consts::PI;

use bevy_math::{Quat, Vec4};
use rand::Rng;

pub trait Random {
    fn random() -> Self;
}

impl Random for Quat {
    fn random() -> Quat {
        let mut rng = rand::thread_rng();
        let u: f32 = rng.gen_range(0.0..1.0);
        let v: f32 = rng.gen_range(0.0..1.0);
        let w: f32 = rng.gen_range(0.0..1.0);

        Quat::from_vec4(Vec4::new(
            (1. - u).sqrt() * (2. * PI * v).sin(),
            (1. - u).sqrt() * (2. * PI * v).cos(),
            u.sqrt() * (2. * PI * w).sin(),
            u.sqrt() * (2. * PI * w).cos(),
        ))
    }
}
