use std::{net::UdpSocket, time::SystemTime};

use app::{controls::player_input, utils::LerpTransformTarget};
use bevy::{
    app::App,
    prelude::{Commands, EventWriter, IntoSystemDescriptor, Plugin, Res, ResMut, Transform},
    utils::HashMap,
};

use bevy_renet::{
    renet::{ClientAuthentication, DefaultChannel, RenetClient, RenetConnectionConfig},
    run_if_client_connected, RenetClientPlugin,
};

use spaaaace_shared::{
    player::player_input::PlayerInput, ClientMessages, Lobby, ServerMessages, TranslationRotation,
    PROTOCOL_ID, SERVER_TICKRATE,
};

pub struct ClientNetworkingPlugin;

impl Plugin for ClientNetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(RenetClientPlugin::default())
            .insert_resource(new_renet_client())
            .insert_resource(PlayerInput::default())
            .add_system(player_input)
            .add_system(client_send_input.with_run_criteria(run_if_client_connected))
            .add_system(client_reliable_message_handler.with_run_criteria(run_if_client_connected))
            .add_system(
                client_unreliable_message_handler.with_run_criteria(run_if_client_connected),
            );
    }
}

fn new_renet_client() -> RenetClient {
    let server_addr = "127.0.0.1:5000".parse().unwrap();
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let connection_config = RenetConnectionConfig::default();
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let client_id = current_time.as_millis() as u64;
    let authentication = ClientAuthentication::Unsecure {
        client_id,
        protocol_id: PROTOCOL_ID,
        server_addr,
        user_data: None,
    };
    RenetClient::new(current_time, socket, connection_config, authentication).unwrap()
}

fn client_send_input(player_input: Res<PlayerInput>, mut client: ResMut<RenetClient>) {
    let client_message = ClientMessages::PlayerInput {
        input: *player_input,
    };
    let input_message = bincode::serialize(&client_message).unwrap();
    client.send_message(DefaultChannel::Reliable, input_message);
}

fn client_reliable_message_handler(
    mut commands: Commands,
    mut client: ResMut<RenetClient>,
    mut lobby: ResMut<Lobby>,
    mut server_message_event_writer: EventWriter<ServerMessages>,
) {
    while let Some(message) = client.receive_message(DefaultChannel::Reliable) {
        let server_message = bincode::deserialize(&message).unwrap();
        server_message_event_writer.send(bincode::deserialize(&message).unwrap());

        match server_message {
            ServerMessages::EntityDespawn { id } => {
                if let Some(entity) = lobby.networked_entities.remove(&id) {
                    commands.entity(entity).despawn();
                }
            }

            _ => (),
        }
    }
}

fn client_unreliable_message_handler(
    mut commands: Commands,
    mut client: ResMut<RenetClient>,
    lobby: ResMut<Lobby>,
) {
    while let Some(message) = client.receive_message(DefaultChannel::Unreliable) {
        let networked_translation: HashMap<u64, TranslationRotation> =
            bincode::deserialize(&message).unwrap();

        for (id, translation_rotation) in networked_translation.iter() {
            if let Some(entity) = lobby.players.get(id) {
                commands.entity(*entity).insert(LerpTransformTarget {
                    target: Transform {
                        translation: translation_rotation.translation,
                        rotation: translation_rotation.rotation,
                        ..Default::default()
                    },
                    speed: SERVER_TICKRATE / 1.2,
                });
            }
        }

        for (id, translation_rotation) in networked_translation.iter() {
            if let Some(entity) = lobby.networked_entities.get(id) {
                commands.entity(*entity).insert(LerpTransformTarget {
                    target: Transform {
                        translation: translation_rotation.translation,
                        rotation: translation_rotation.rotation,
                        ..Default::default()
                    },
                    speed: SERVER_TICKRATE / 1.2,
                });
            }
        }
    }
}
