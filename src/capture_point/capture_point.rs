use bevy::{
    prelude::{Bundle, Component, PbrBundle, Query, Res, Transform, With},
    time::Time,
};

use crate::ship::space_ship::SpaceShip;

#[derive(Component)]
pub struct CaptureSphere {
    radius: f32,
    progress: f32,
    attacker: u8,
}

#[derive(Bundle)]
pub struct CapturePoint {
    pub capture: CaptureSphere,
    pub pbr_bundle: PbrBundle,
}

pub fn capture_system(
    mut query_capture_spheres: Query<(&Transform, &mut CaptureSphere)>,
    query_space_ship: Query<&Transform, With<SpaceShip>>,
    time: Res<Time>,
) {
    for ship_transform in query_space_ship.iter() {
        for (capture_transform, mut capture_sphere) in query_capture_spheres.iter_mut() {
            let distance = ship_transform
                .translation
                .distance(capture_transform.translation);
            let capture_speed = 1.0;

            if capture_sphere.radius < distance {
                capture_sphere.progress += capture_speed * time.delta_seconds();

                print!("{}", capture_sphere.progress);
            }
        }
    }
}
