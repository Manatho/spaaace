use bevy::prelude::{
    App, Color, Input, KeyCode, MouseButton, Plugin, Query, Res, ResMut, Transform, Vec3, With,
};
use bevy_mod_gizmos::{draw_gizmo, Gizmo};
use bevy_rapier3d::prelude::{QueryFilter, RapierContext};
use spaaaace_shared::{player::player_input::PlayerInput, targeting::Targetable, NetworkedId};

use crate::camera::{OrbitCamera, OrbitCameraTarget};

pub struct ControlsPlugin;

impl Plugin for ControlsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(player_input)
            .add_system(targetting)
            .add_system(aiming);
    }
}

pub fn player_input(
    k_input: Res<Input<KeyCode>>,
    m_input: Res<Input<MouseButton>>,
    mut player_input: ResMut<PlayerInput>,
) {
    player_input.rotate_left = k_input.pressed(KeyCode::A);
    player_input.rotate_right = k_input.pressed(KeyCode::D);
    player_input.thrust_forward = k_input.pressed(KeyCode::W);
    player_input.thrust_reverse = k_input.pressed(KeyCode::S);
    player_input.thrust_left = k_input.pressed(KeyCode::Q);
    player_input.thrust_right = k_input.pressed(KeyCode::E);
    player_input.thrust_up = k_input.pressed(KeyCode::Space);
    player_input.thrust_down = k_input.pressed(KeyCode::LControl);
    player_input.ability_slot_1 = k_input.pressed(KeyCode::Key1);
    player_input.ability_slot_2 = k_input.pressed(KeyCode::Key2);
    player_input.ability_slot_3 = k_input.pressed(KeyCode::Key3);
    player_input.ability_slot_4 = k_input.pressed(KeyCode::Key4);
    player_input.ability_slot_5 = k_input.pressed(KeyCode::Key5);
    player_input.ability_slot_6 = k_input.pressed(KeyCode::Key6);
    player_input.ability_slot_7 = k_input.pressed(KeyCode::Key7);
    player_input.ability_slot_8 = k_input.pressed(KeyCode::Key8);
    player_input.ability_slot_9 = k_input.pressed(KeyCode::Key9);
    player_input.primary_fire = m_input.pressed(MouseButton::Left);
}

fn aiming(
    camera_query: Query<&Transform, With<OrbitCamera>>,
    camera_target_query: Query<&Transform, With<OrbitCameraTarget>>,
    rapier_context: Res<RapierContext>,
    mut player_input: ResMut<PlayerInput>,
) {
    match camera_query.get_single() {
        Ok(transform) => {
            let target_transform_result = camera_target_query.get_single();
            match target_transform_result {
                Ok(_) => {
                    let max_toi = 1000000.0;
                    let solid = true;
                    let ray_pos = transform.translation;
                    let ray_dir = transform.forward();
                    let filter = QueryFilter::default();

                    let mut hit_point = ray_pos + ray_dir * max_toi;
                    if let Some((_, toi)) =
                        rapier_context.cast_ray(ray_pos, ray_dir, max_toi, solid, filter)
                    {
                        hit_point = ray_pos + ray_dir * toi;
                    }
                    player_input.aim_point = hit_point;
                }
                Err(_) => (),
            }

            draw_gizmo(Gizmo::new(player_input.aim_point, 1.0, Color::GREEN));
        }
        Err(_) => {}
    }
}

fn targetting(
    keys: Res<Input<KeyCode>>,
    target_query: Query<(&Transform, &NetworkedId), With<Targetable>>,
    camera_query: Query<&Transform, With<OrbitCamera>>,
    mut player_input: ResMut<PlayerInput>,
) {
    if keys.just_pressed(KeyCode::T) {
        match camera_query.get_single() {
            Ok(camera) => {
                let mut min_distance = f32::MAX;
                let mut min_id: u64 = 0;

                for (transform, network_id) in target_query.into_iter() {
                    let distance = nearest_point_on_line_to_point(
                        camera.translation,
                        camera.forward(),
                        transform.translation,
                    )
                    .length_squared();

                    if distance < min_distance {
                        min_distance = distance;
                        min_id = network_id.id;
                    }
                }
                player_input.target_network_id = min_id;
            }
            Err(_) => todo!(),
        }
    }
}

fn nearest_point_on_line_to_point(origin: Vec3, direction: Vec3, point: Vec3) -> Vec3 {
    let point_to_origin = origin - point;
    let point_to_closest_point_on_line =
        point_to_origin - point_to_origin.dot(direction) * direction;
    return point_to_closest_point_on_line;
}
