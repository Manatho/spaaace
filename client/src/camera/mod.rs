use std::f32::consts::PI;

use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::{
        App, Component, EventReader, Plugin, Quat, Query, Res, Transform, Vec2, Vec3, With, Without,
    },
    window::Windows,
};

pub struct OrbitCameraPlugin;

impl Plugin for OrbitCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(camera_follow_local_player);
    }
}

#[derive(Component)]
pub struct OrbitCameraTarget;

#[derive(Component)]
pub struct OrbitCamera {
    pub zoom: f32,
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
                    + Vec3::Y * orbit_camera.zoom / 4.0
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
