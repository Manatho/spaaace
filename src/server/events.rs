use bevy::prelude::{info, EventReader, ResMut};
use naia_bevy_server::{
    events::{AuthorizationEvent, ConnectionEvent, DisconnectionEvent, MessageEvent},
    shared::Random,
    Server,
};

use crate::networking::{
    channels::Channels,
    protocol::{EntityAssignment, NetworkPosition, Protocol},
};

use super::resources::Global;

pub fn authorization_event(
    mut event_reader: EventReader<AuthorizationEvent<Protocol>>,
    mut server: Server<Protocol, Channels>,
) {
    for event in event_reader.iter() {
        if let AuthorizationEvent(user_key, Protocol::Auth(auth)) = event {
            server.accept_connection(user_key);
        }
    }
}

pub fn connection_event<'world, 'state>(
    mut event_reader: EventReader<ConnectionEvent>,
    mut global: ResMut<Global>,
    mut server: Server<'world, 'state, Protocol, Channels>,
) {
    for event in event_reader.iter() {
        let ConnectionEvent(user_key) = event;
        let address = server
            .user_mut(user_key)
            // Add User to the main Room
            .enter_room(&global.main_room_key)
            // Get User's address for logging
            .address();

        info!("Naia Server connected to: {}", address);

        // Create components for Entity to represent new player

        // Position component
        let position = {
            let x = 0.;
            let y = 0.;
            let z = 0.;
            NetworkPosition::new(x, y, z)
        };

        // Spawn entity
        let entity = server
            // Spawn new Square Entity
            .spawn()
            // Add Entity to main Room
            .enter_room(&global.main_room_key)
            // Insert Position component
            .insert(position)
            // return Entity id
            .id();

        global.user_to_prediction_map.insert(*user_key, entity);

        // Send an Entity Assignment message to the User that owns the Square
        let mut assignment_message = EntityAssignment::new(true);
        assignment_message.entity.set(&server, &entity);

        server.send_message(user_key, Channels::EntityAssignment, &assignment_message);
    }
}

pub fn disconnection_event(
    mut event_reader: EventReader<DisconnectionEvent>,
    mut global: ResMut<Global>,
    mut server: Server<Protocol, Channels>,
) {
    for event in event_reader.iter() {
        let DisconnectionEvent(user_key, user) = event;
        info!("Naia Server disconnected from: {:?}", user.address);

        if let Some(entity) = global.user_to_prediction_map.remove(user_key) {
            server
                .entity_mut(&entity)
                .leave_room(&global.main_room_key)
                .despawn();
        }
    }
}

pub fn receive_message_event(
    mut event_reader: EventReader<MessageEvent<Protocol, Channels>>,
    mut global: ResMut<Global>,
    server: Server<Protocol, Channels>,
) {
    for event in event_reader.iter() {
        if let MessageEvent(_user_key, Channels::PlayerCommand, Protocol::KeyCommand(key_command)) =
            event
        {
            if let Some(entity) = &key_command.entity.get(&server) {
                global
                    .player_last_command
                    .insert(*entity, key_command.clone());
            }
        }
    }
}
