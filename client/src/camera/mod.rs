use std::f32::consts::PI;

use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::{
        App, Component, EventReader, Plugin, Quat, Query, Res, SystemSet, Transform, Vec2, Vec3,
        With, Without,
    },
    window::Windows,
};

use crate::game_state::run_if_not_paused;

pub struct OrbitCameraPlugin;

impl Plugin for OrbitCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::new()
                .with_system(camera_follow_local_player)
                .with_run_criteria(run_if_not_paused),
        );
    }
}

#[derive(Component)]
pub struct OrbitCameraTarget;

#[derive(Component)]
pub struct OrbitCamera {
    pub zoom: f32,
    pub offset: Vec3,
}

fn camera_follow_local_player(
    mut camera_query: Query<(&mut Transform, &mut OrbitCamera), Without<OrbitCameraTarget>>,
    local_player_query: Query<&Transform, With<OrbitCameraTarget>>,
    mut motion_evr: EventReader<MouseMotion>,
    mut scroll_evr: EventReader<MouseWheel>,
    windows: Res<Windows>,
) {
    let mut rotation_move = Vec2::ZERO;

    for ev in motion_evr.iter() {
        rotation_move += ev.delta * 10.0;
    }

    let scroll_zoom = scroll_evr.iter().map(|x| -x.y * 6.0).sum::<f32>();

    for (mut transform, mut orbit_camera) in camera_query.iter_mut() {
        orbit_camera.zoom += scroll_zoom;
        match local_player_query.get_single() {
            Ok(local_player_transform) => {
                if rotation_move.length_squared() > 0.0 {
                    let window = get_primary_window_size(&windows);
                    let delta = (rotation_move / window) / PI;
                    let yaw = Quat::from_rotation_y(-delta.x);
                    let pitch = Quat::from_rotation_x(-delta.y);
                    transform.rotation = yaw * transform.rotation; // rotate around global y axis
                    transform.rotation = transform.rotation * pitch; // rotate around local x axis
                }
                transform.translation = local_player_transform.translation
                    + orbit_camera.offset
                    + transform.back() * orbit_camera.zoom;
                // transform.rotation *= Rot
            }
            Err(_) => {}
        }
    }
}

fn get_primary_window_size(windows: &Res<Windows>) -> Vec2 {
    let window = windows.get_primary().unwrap();
    let window = Vec2::new(window.width() as f32, window.height() as f32);
    window
}
