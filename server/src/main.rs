use std::{net::UdpSocket, time::SystemTime};

use bevy::{
    app::ScheduleRunnerPlugin,
    log::LogPlugin,
    prelude::{
        info, App, Commands, Component, CorePlugin, CoreStage, EventReader, GlobalTransform, Query,
        Res, ResMut, StageLabel, SystemStage, Transform,
    },
    time::{FixedTimestep, Time, TimePlugin},
    transform::TransformBundle,
    utils::HashMap,
};

use bevy_renet::{
    renet::{
        DefaultChannel, RenetConnectionConfig, RenetServer, ServerAuthentication, ServerConfig,
        ServerEvent,
    },
    RenetServerPlugin,
};

use spaaaace_shared::{Lobby, PlayerInput, ServerMessages, PROTOCOL_ID};

use crate::projectiles::ProjectilesPlugin;

mod networking;
mod projectiles;

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
struct FixedUpdateStage;

#[derive(Debug, Component)]
struct Player {
    id: u64,
}

const PLAYER_MOVE_SPEED: f32 = 2.0;

fn main() {
    info!("Naia Bevy Server Demo starting up");

    // Build App
    App::default()
        // Plugins
        .insert_resource(Lobby::default())
        .add_plugin(CorePlugin::default())
        .add_plugin(TimePlugin::default())
        .add_plugin(ScheduleRunnerPlugin::default())
        .add_plugin(LogPlugin::default())
        .add_plugin(RenetServerPlugin::default())
        .insert_resource(new_renet_server())
        .add_system(server_update_system)
        .add_system(update_players_system)
        .add_plugin(ProjectilesPlugin {})
        .add_stage_after(
            CoreStage::Update,
            FixedUpdateStage,
            SystemStage::parallel()
                .with_run_criteria(FixedTimestep::step(1.0 / 30.0))
                .with_system(server_sync_players),
        )
        // Run App
        .run();
}

fn new_renet_server() -> RenetServer {
    let server_addr = "127.0.0.1:5000".parse().unwrap();
    let socket = UdpSocket::bind(server_addr).unwrap();
    let connection_config = RenetConnectionConfig::default();
    let server_config =
        ServerConfig::new(64, PROTOCOL_ID, server_addr, ServerAuthentication::Unsecure);
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    RenetServer::new(current_time, server_config, connection_config, socket).unwrap()
}

fn server_update_system(
    mut server_events: EventReader<ServerEvent>,
    mut commands: Commands,
    mut lobby: ResMut<Lobby>,
    mut server: ResMut<RenetServer>,
) {
    for event in server_events.iter() {
        match event {
            ServerEvent::ClientConnected(id, _) => {
                println!("Player {} connected.", id);
                // Spawn player cube
                let player_entity = commands
                    .spawn(TransformBundle {
                        global: GlobalTransform::from_xyz(0.0, 0.5, 0.0),
                        ..Default::default()
                    })
                    .insert(PlayerInput::default())
                    .insert(Player { id: *id })
                    .id();

                // We could send an InitState with all the players id and positions for the client
                // but this is easier to do.
                for &player_id in lobby.players.keys() {
                    let message =
                        bincode::serialize(&ServerMessages::PlayerConnected { id: player_id })
                            .unwrap();
                    server.send_message(*id, DefaultChannel::Reliable, message);
                }

                lobby.players.insert(*id, player_entity);

                let message =
                    bincode::serialize(&ServerMessages::PlayerConnected { id: *id }).unwrap();
                server.broadcast_message(DefaultChannel::Reliable, message);
            }
            ServerEvent::ClientDisconnected(id) => {
                println!("Player {} disconnected.", id);
                if let Some(player_entity) = lobby.players.remove(id) {
                    commands.entity(player_entity).despawn();
                }

                let message =
                    bincode::serialize(&ServerMessages::PlayerDisconnected { id: *id }).unwrap();
                server.broadcast_message(DefaultChannel::Reliable, message);
            }
        }
    }

    for client_id in server.clients_id().into_iter() {
        while let Some(message) = server.receive_message(client_id, DefaultChannel::Reliable) {
            let player_input: PlayerInput = bincode::deserialize(&message).unwrap();
            if let Some(player_entity) = lobby.players.get(&client_id) {
                commands.entity(*player_entity).insert(player_input);
            }
        }
    }
}

fn server_sync_players(mut server: ResMut<RenetServer>, query: Query<(&Transform, &Player)>) {
    let mut players: HashMap<u64, [f32; 3]> = HashMap::new();
    for (transform, player) in query.iter() {
        players.insert(player.id, transform.translation.into());
    }

    let sync_message = bincode::serialize(&players).unwrap();
    server.broadcast_message(DefaultChannel::Unreliable, sync_message);
}

fn update_players_system(
    mut query: Query<(&mut Transform, &PlayerInput)>,
    time: Res<Time>,
    mut commands: Commands,
    mut server: ResMut<RenetServer>,
) {
    for (mut transform, input) in query.iter_mut() {
        let x = (input.right as i8 - input.left as i8) as f32;
        let y = (input.down as i8 - input.up as i8) as f32;
        transform.translation.x += x * PLAYER_MOVE_SPEED * time.delta().as_secs_f32();
        transform.translation.z += y * PLAYER_MOVE_SPEED * time.delta().as_secs_f32();

        if input.primary_fire {
            commands.spawn(TransformBundle::from_transform(Transform {
                translation: transform.translation,
                ..Default::default()
            }));
            let message = bincode::serialize(&ServerMessages::BulletSpawned {
                position: transform.translation,
                rotation: transform.rotation,
            })
            .unwrap();
            server.broadcast_message(DefaultChannel::Reliable, message);
        }
    }
}
