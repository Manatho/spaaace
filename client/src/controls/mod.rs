use bevy::{
    math::vec2,
    prelude::{Input, KeyCode, Res, ResMut},
};
use spaaaace_shared::player::player_input::PlayerInput;

pub fn player_input(k_input: Res<Input<KeyCode>>, mut player_input: ResMut<PlayerInput>) {
    player_input.rotate_left = k_input.pressed(KeyCode::A);
    player_input.rotate_right = k_input.pressed(KeyCode::D);
    player_input.thrust_forward = k_input.pressed(KeyCode::W);
    player_input.thrust_reverse = k_input.pressed(KeyCode::S);
    player_input.thrust_left = k_input.pressed(KeyCode::Q);
    player_input.thrust_right = k_input.pressed(KeyCode::E);
    player_input.thrust_up = k_input.pressed(KeyCode::Space);
    player_input.thrust_down = k_input.pressed(KeyCode::LControl);
    player_input.primary_fire = k_input.pressed(KeyCode::Return);
    player_input.aim_dir = vec2(0.0, 0.0);
}
