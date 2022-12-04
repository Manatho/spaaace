use bevy_math::{vec3};

use crate::protocol::{KeyCommand, Position};

pub fn process_command(key_command: &KeyCommand, position: &mut Position) {
    let input_vector = vec3(
        *key_command.right as i32 as f32 - *key_command.left as i32 as f32,
        0.,
        *key_command.forward as i32 as f32 - *key_command.backward as i32 as f32,
    )
    .normalize_or_zero();
    *position += input_vector * 0.1;
}
