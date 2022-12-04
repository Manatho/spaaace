use bevy::{
    ecs::system::{Res, ResMut},
    input::{keyboard::KeyCode, Input},
};

use naia_bevy_client::Client;

use spaaaace_shared::{
    protocol::{KeyCommand, Protocol},
    Channels,
};

use crate::resources::Global;

pub fn input(
    mut global: ResMut<Global>,
    client: Client<Protocol, Channels>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    let w = keyboard_input.pressed(KeyCode::W);
    let s = keyboard_input.pressed(KeyCode::S);
    let a = keyboard_input.pressed(KeyCode::A);
    let d = keyboard_input.pressed(KeyCode::D);
    let space = keyboard_input.pressed(KeyCode::Space);

    if let Some(command) = &mut global.queued_command {
        
        *command.forward = w;
        *command.backward = s;
        *command.left = a;
        *command.right = d;
        *command.primary_fire = space;

    } else if let Some(owned_entity) = &global.owned_entity {
        let mut key_command = KeyCommand::new(w, s, a, d, space);
        key_command.entity.set(&client, &owned_entity.confirmed);
        global.queued_command = Some(key_command);
    }
}
