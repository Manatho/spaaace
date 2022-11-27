use bevy::{
    prelude::{Component, Input, KeyCode, Query, Res, Transform},
    time::Time,
};

#[derive(Component)]
pub struct SpaceShip {
    pub hp: i32,
}

pub fn sometimes_move(
    mut query: Query<(&mut Transform, &SpaceShip)>,
    time: Res<Time>,
    keyboard: Res<Input<KeyCode>>,
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
    }
}
