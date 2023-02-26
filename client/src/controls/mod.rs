use bevy::prelude::{
    Color, Component, Input, KeyCode, MouseButton, Query, Res, ResMut, Transform, With,
};
use bevy_mod_gizmos::{draw_gizmo, Gizmo};
use spaaaace_shared::player::player_input::PlayerInput;

use crate::camera::{OrbitCamera, OrbitCameraTarget};

pub fn player_input(
    k_input: Res<Input<KeyCode>>,
    m_input: Res<Input<MouseButton>>,
    mut player_input: ResMut<PlayerInput>,
    camera_query: Query<(&Transform, &OrbitCamera)>,
    camera_target_query: Query<&Transform, With<OrbitCameraTarget>>,
) {
    player_input.rotate_left = k_input.pressed(KeyCode::A);
    player_input.rotate_right = k_input.pressed(KeyCode::D);
    player_input.thrust_forward = k_input.pressed(KeyCode::W);
    player_input.thrust_reverse = k_input.pressed(KeyCode::S);
    player_input.thrust_left = k_input.pressed(KeyCode::Q);
    player_input.thrust_right = k_input.pressed(KeyCode::E);
    player_input.thrust_up = k_input.pressed(KeyCode::Space);
    player_input.thrust_down = k_input.pressed(KeyCode::LControl);
    player_input.primary_fire = m_input.pressed(MouseButton::Left);

    match camera_query.get_single() {
        Ok((transform, orbit_camera)) => {
            let target_transform_result = camera_target_query.get_single();
            match target_transform_result {
                Ok(target_transform) => {
                    player_input.aim_point = target_transform.translation
                        + orbit_camera.offset
                        + (transform.forward() * 50.0)
                }
                Err(_) => (),
            }

            draw_gizmo(Gizmo::new(player_input.aim_point, 1.0, Color::GREEN));
        }
        Err(_) => {}
    }
}

#[derive(Component)]
pub struct LocalPlayer;
pub fn local_player_input_sync(
    mut query: Query<(&LocalPlayer, &mut PlayerInput)>,
    player_input: Res<PlayerInput>,
) {
    for (_, mut input) in query.iter_mut() {
        input.aim_point = player_input.aim_point;
    }
}
