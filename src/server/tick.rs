use bevy::{
    prelude::{Query, Res, ResMut, Without},
    time::Time,
};
use naia_bevy_server::Server;

use crate::{
    client::events::ClientSide,
    networking::{
        behavior::process_command,
        channels::Channels,
        protocol::{NetworkPosition, Protocol},
    },
};

use super::resources::Global;

pub fn tick(
    mut global: ResMut<Global>,
    time: Res<Time>,
    mut server: Server<Protocol, Channels>,
    mut position_query: Query<&mut NetworkPosition, Without<ClientSide>>,
) {
    // All game logic should happen here, on a tick event
    //info!("tick");

    // Update scopes of entities
    for (_, user_key, entity) in server.scope_checks() {
        // You'd normally do whatever checks you need to in here..
        // to determine whether each Entity should be in scope or not.

        // This indicates the Entity should be in this scope.
        server.user_scope(&user_key).include(&entity);

        // And call this if Entity should NOT be in this scope.
        // server.user_scope(..).exclude(..);
    }

    // Process all received commands
    for (entity, last_command) in global.player_last_command.drain() {
        if let Ok(mut position) = position_query.get_mut(entity) {
            process_command(&last_command, &mut position, &time);
        }
    }

    // This is very important! Need to call this to actually send all update packets
    // to all connected Clients!
    server.send_all_updates();
}
