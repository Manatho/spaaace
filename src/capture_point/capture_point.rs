use bevy::{
    prelude::{
        AlphaMode, Bundle, Component, Material, PbrBundle, Query, Res, Transform, Vec4, With,
    },
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef},
    time::Time,
};

use crate::ship::space_ship::SpaceShip;

#[derive(Component)]
pub struct CaptureSphere {
    pub radius: f32,
    pub progress: f32,
    pub attacker: u8,
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

// This is the struct that will be passed to your shader
#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "f690fdae-d598-45ab-8225-97e2a3f056e0"]
pub struct ForceFieldMaterial {
    // #[uniform(0)]
    // pub selection: Vec4,
}

impl Material for ForceFieldMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/forcefield.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}
