use bevy::{
    prelude::{
        Assets, Commands, Component, Input, KeyCode, Mesh, Query, Res, ResMut, StandardMaterial,
        Transform,
    },
    time::Time,
};

use crate::projectile::projectile::LaserProjectileBundle;

#[derive(Component)]
pub struct SpaceShip {
    pub hp: i32,
}

pub fn forever_move(mut query: Query<(&mut Transform, &SpaceShip)>, time: Res<Time>) {
    for (mut transform, _) in query.iter_mut() {
        let forward = transform.forward().clone();
        transform.translation += forward * time.delta_seconds();
    }
}

pub fn sometimes_move(
    mut query: Query<(&mut Transform, &SpaceShip)>,
    mut commands: Commands,
    time: Res<Time>,
    keyboard: Res<Input<KeyCode>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (mut transform, _) in query.iter_mut() {
        let forward = transform.forward().clone();
        let right = transform.right().clone();
        if keyboard.pressed(KeyCode::W) {
            transform.translation += forward * time.delta_seconds();
        }
        if keyboard.pressed(KeyCode::S) {
            transform.translation -= forward * time.delta_seconds();
        }
        if keyboard.pressed(KeyCode::D) {
            transform.translation += right * time.delta_seconds();
        }
        if keyboard.pressed(KeyCode::A) {
            transform.translation -= right * time.delta_seconds();
        }
        if (keyboard.pressed(KeyCode::Space)) {
            commands.spawn(LaserProjectileBundle::new(meshes, materials));
        }
    }
}
