use std::{net::UdpSocket, time::SystemTime};

use bevy::{
    math::vec3,
    prelude::{
        default, info, App, BuildChildren, Camera3dBundle, Color, Commands, Component, CoreStage,
        DespawnRecursiveExt, EventReader, EventWriter, PluginGroup, Quat, Query, Res, ResMut,
        StageLabel, SystemStage, Transform, Vec3,
    },
    time::{FixedTimestep, Time},
    transform::TransformBundle,
    utils::HashMap,
    window::{PresentMode, WindowDescriptor, WindowPlugin},
    DefaultPlugins,
};

use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_mod_gizmos::{draw_gizmo, Gizmo, GizmosPlugin};
use bevy_rapier3d::{
    prelude::{
        Collider, Damping, ExternalForce, GravityScale, NoUserData, RapierPhysicsPlugin, RigidBody,
    },
    render::RapierDebugRenderPlugin,
};
use bevy_renet::{
    renet::{
        DefaultChannel, RenetConnectionConfig, RenetServer, ServerAuthentication, ServerConfig,
        ServerEvent,
    },
    RenetServerPlugin,
};

use capture_point::capture_point::CaptureSphere;

use player::Player;
use spaaaace_shared::{
    team::team_enum::Team, ClientMessages, Lobby, PlayerInput, ServerMessages, TranslationRotation,
    PROTOCOL_ID, SERVER_TICKRATE,
};

use crate::{
    capture_point::CapturePointPlugin,
    player::PlayerPlugin,
    weapons::{Turret, WeaponsPlugin},
};

pub mod capture_point;
pub mod player;
mod weapons;

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
struct FixedUpdateStage;

fn main() {
    info!("Naia Bevy Server Demo starting up");

    // Build App
    App::default()
        // Plugins
        .insert_resource(Lobby::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "Spaaace Server".to_string(),
                width: 320.,
                height: 240.,
                present_mode: PresentMode::AutoVsync,
                ..default()
            },
            ..default()
        }))
        .add_plugin(GizmosPlugin)
        .add_startup_system(init)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(RenetServerPlugin::default())
        .insert_resource(new_renet_server())
        .add_system(server_update_system)
        .add_plugin(WeaponsPlugin {})
        .add_plugin(PlayerPlugin)
        .add_plugin(CapturePointPlugin)
        .add_event::<ClientMessages>()
        .add_system(client_connected)
        .add_system(client_disconnected)
        // Server UI for debugging
        // .add_plugin(InputPlugin::default())
        // .add_plugin(ScenePlugin::default())
        // .add_plugin(WindowPlugin::default())
        // .add_plugin(WinitPlugin::default())
        // .add_plugin(RenderPlugin::default())
        // Run App
        .run();
}

fn init(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 6., -12.0).looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
        ..default()
    });
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

fn client_connected(
    mut event_reader: EventReader<ServerEvent>,
    mut commands: Commands,
    mut lobby: ResMut<Lobby>,
    mut server: ResMut<RenetServer>,
    capture_point_query: Query<(&Transform, &CaptureSphere)>,
) {
    for event in event_reader.iter() {
        match event {
            ServerEvent::ClientConnected(id, _) => {
                println!("Player {} connected.", id);
                // Spawn player cube
                let player_entity = commands
                    .spawn(TransformBundle {
                        local: Transform {
                            translation: vec3(0.0, 0.5, 0.0),
                            rotation: Quat::from_rotation_x(0.5),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .insert(PlayerInput::default())
                    .insert(Player {
                        id: *id,
                        team: Team::Red,
                    })
                    .insert(Collider::cuboid(1.0, 1.0, 1.0))
                    .insert(RigidBody::Dynamic)
                    // .insert(LockedAxes::ROTATION_LOCKED_Z)
                    .insert(GravityScale(0.0))
                    .insert(ExternalForce::default())
                    .insert(Damping {
                        linear_damping: 0.5,
                        angular_damping: 1.0,
                    })
                    .with_children(|parent| {
                        println!("spawning turret");
                        parent
                            .spawn(TransformBundle {
                                ..Default::default()
                            })
                            .insert(Turret {
                                cooldown: 0.0,
                                fire_rate: 1.0 / 5.0,
                                trigger: false,
                            });
                    })
                    .id();

                // We could send an InitState with all the players id and positions for the client
                // but this is easier to do.
                for &player_id in lobby.players.keys() {
                    let message =
                        bincode::serialize(&ServerMessages::PlayerConnected { id: player_id })
                            .unwrap();
                    server.send_message(*id, DefaultChannel::Reliable, message);
                }

                for (&transform, capture_point) in capture_point_query.iter() {
                    let message = bincode::serialize(&ServerMessages::CapturePointSpawned {
                        position: transform.translation,
                        rotation: transform.rotation,
                        id: capture_point.id,
                        owner: capture_point.owner.clone(),
                        progress: capture_point.progress,
                    })
                    .unwrap();
                    server.send_message(*id, DefaultChannel::Reliable, message);
                }

                lobby.players.insert(*id, player_entity);

                let message =
                    bincode::serialize(&ServerMessages::PlayerConnected { id: *id }).unwrap();
                server.broadcast_message(DefaultChannel::Reliable, message);
            }
            _ => (),
        }
    }
}

fn client_disconnected(
    mut event_reader: EventReader<ServerEvent>,
    mut commands: Commands,
    mut lobby: ResMut<Lobby>,
    mut server: ResMut<RenetServer>,
) {
    for event in event_reader.iter() {
        match event {
            ServerEvent::ClientDisconnected(id) => {
                println!("Player {} disconnected.", id);
                if let Some(player_entity) = lobby.players.remove(id) {
                    commands.entity(player_entity).despawn_recursive();
                }

                let message =
                    bincode::serialize(&ServerMessages::PlayerDisconnected { id: *id }).unwrap();
                server.broadcast_message(DefaultChannel::Reliable, message);
            }
            _ => (),
        }
    }
}

fn server_update_system(
    mut commands: Commands,
    lobby: ResMut<Lobby>,
    mut server: ResMut<RenetServer>,
    mut player_query: Query<&mut Player>,
) {
    for client_id in server.clients_id().into_iter() {
        while let Some(message) = server.receive_message(client_id, DefaultChannel::Reliable) {
            let client_message: ClientMessages = bincode::deserialize(&message).unwrap();
            match client_message {
                ClientMessages::PlayerInput { input } => {
                    if let Some(player_entity) = lobby.players.get(&client_id) {
                        commands.entity(*player_entity).insert(input);
                    }
                }
                ClientMessages::Command { command } => {
                    let args_split = command.split(" ");
                    let args: Vec<&str> = args_split.collect();

                    match args[0] {
                        "swap_team" => {
                            let entity = lobby.players[&client_id];

                            match player_query.get_mut(entity) {
                                Ok(mut player) => match args[1] {
                                    "1" => player.team = Team::Red,
                                    "2" => player.team = Team::Blue,
                                    _ => (),
                                },
                                Err(_) => (),
                            }
                        }
                        _ => (),
                    }
                }
            }
        }
    }
}
