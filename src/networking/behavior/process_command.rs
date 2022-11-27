use bevy::{time::{self, Time}, prelude::{ResMut, Res}};

use crate::networking::protocol::{KeyCommand, NetworkPosition};

const SQUARE_SPEED: f32 = 1.0;

pub fn process_command(
    key_command: &KeyCommand,
    position: &mut NetworkPosition,
    time: &Res<Time>,
) {
    if *key_command.w {
        *position.y += SQUARE_SPEED * time.delta_seconds();
    }
    if *key_command.s {
        *position.y -= SQUARE_SPEED * time.delta_seconds();
    }
    if *key_command.a {
        *position.x += SQUARE_SPEED * time.delta_seconds();
    }
    if *key_command.d {
        *position.x -= SQUARE_SPEED * time.delta_seconds();
    }
}
