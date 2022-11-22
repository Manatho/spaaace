use bevy::prelude::{
    default, shape, Assets, Bundle, Color, Component, Mesh, PbrBundle, Query, ResMut,
    StandardMaterial, Transform,
};

#[derive(Component)]
pub struct Projectile {
    pub velocity: f32,
}

#[derive(Bundle)]
pub struct LaserProjectileBundle {
    pub projectile: Projectile,
    pub pbr_bundle: PbrBundle,
}

impl LaserProjectileBundle {
    pub fn new(
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) -> LaserProjectileBundle {
        Self {
            projectile: Projectile { velocity: 1. },
            pbr_bundle: PbrBundle {
                mesh: meshes.add(shape::Cube { size: 1. }.into()),
                material: materials.add(Color::GREEN.into()),
                ..default()
            },
        }
    }
}

pub fn move_projectile_system(mut query: Query<(&mut Transform, &Projectile)>) {
    for (mut transform, projectil) in query.iter_mut() {}
}
