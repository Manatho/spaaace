use bevy::{ecs::system::{Query, ResMut}, time::Time, prelude::Res};

use naia_bevy_client::Client;

use crate::networking::{protocol::{Protocol, NetworkPosition}, channels::Channels, behavior::process_command};

use super::global::ClientGlobal;

pub fn tick(
    mut global: ResMut<ClientGlobal>,
    time: Res<Time>,
    mut client: Client<Protocol, Channels>,
    mut position_query: Query<&mut NetworkPosition>,
) {
    //All game logic should happen here, on a tick event

    if let Some(command) = global.queued_command.take() {
        if let Some(predicted_entity) = global
            .owned_entity
            .as_ref()
            .map(|owned_entity| owned_entity.predicted)
        {
            if let Some(client_tick) = client.client_tick() {
                if global.command_history.can_insert(&client_tick) {
                    // Record command
                    global.command_history.insert(client_tick, command.clone());

                    // Send command
                    client.send_message(Channels::PlayerCommand, &command);

                    // Apply command
                    if let Ok(mut position) = position_query.get_mut(predicted_entity) {
                        process_command(&command, &mut position, &time);
                    }
                }
            }
        }
    }
}
