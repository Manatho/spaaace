use std::{net::UdpSocket, time::SystemTime};

use bevy::{
    math::vec3,
    prelude::{
        default, info, App, BuildChildren, Camera3dBundle, Color, Commands, Component, CoreStage,
        EventReader, PluginGroup, Query, Res, ResMut, StageLabel, SystemStage, Transform, Vec3,
        With,
    },
    time::{FixedTimestep, Time},
    transform::TransformBundle,
    utils::HashMap,
    window::{PresentMode, WindowDescriptor, WindowPlugin},
    DefaultPlugins,
};

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

use spaaaace_shared::{
    Lobby, PlayerInput, ServerMessages, TranslationRotation, PROTOCOL_ID, SERVER_TICKRATE,
};

use crate::{
    capture_point::CapturePointPlugin,
    weapons::{Turret, WeaponsPlugin},
};

pub mod capture_point;
mod weapons;

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
struct FixedUpdateStage;

#[derive(Debug, Component)]
pub struct Player {
    id: u64,
}

const PLAYER_MOVE_SPEED: f32 = 2.0;

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
        .add_system(draw_player_gizmos)
        .add_startup_system(init)
        // .add_plugin(CorePlugin::default())
        // .add_plugin(TimePlugin::default())
        // .add_plugin(HierarchyPlugin::default())
        // .add_plugin(ScheduleRunnerPlugin::default())
        // .add_plugin(LogPlugin::default())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(RenetServerPlugin::default())
        .insert_resource(new_renet_server())
        .add_system(server_update_system)
        .add_system(update_players_system)
        .add_plugin(WeaponsPlugin {})
        .add_plugin(CapturePointPlugin)
        .add_stage_after(
            CoreStage::Update,
            FixedUpdateStage,
            SystemStage::parallel()
                .with_run_criteria(FixedTimestep::step(1.0 / (SERVER_TICKRATE as f64)))
                .with_system(server_sync_players),
        )
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

fn draw_player_gizmos(
    query: Query<(&Player, &Transform)>,
    cap_query: Query<(&CaptureSphere, &Transform)>,
) {
    for (_, transform) in query.iter() {
        draw_gizmo(Gizmo::sphere(transform.translation, 1.0, Color::RED))
    }

    for (_, transform) in cap_query.iter() {
        draw_gizmo(Gizmo::sphere(transform.translation, 1.0, Color::GREEN))
    }
}

fn server_update_system(
    mut server_events: EventReader<ServerEvent>,
    mut commands: Commands,
    mut lobby: ResMut<Lobby>,
    mut server: ResMut<RenetServer>,
    mut capture_point_query: Query<&Transform, With<CaptureSphere>>,
) {
    for event in server_events.iter() {
        match event {
            ServerEvent::ClientConnected(id, _) => {
                println!("Player {} connected.", id);
                // Spawn player cube
                let player_entity = commands
                    .spawn(TransformBundle {
                        local: Transform {
                            translation: vec3(0.0, 0.5, 0.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .insert(PlayerInput::default())
                    .insert(Player { id: *id })
                    .insert(Collider::cuboid(1.0, 1.0, 1.0))
                    .insert(RigidBody::Dynamic)
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

                for &capture_point_transform in capture_point_query.iter() {
                    let message = bincode::serialize(&ServerMessages::CapturePointSpawned {
                        position: capture_point_transform.translation,
                        rotation: capture_point_transform.rotation,
                    })
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
    let mut players: HashMap<u64, TranslationRotation> = HashMap::new();
    for (transform, player) in query.iter() {
        players.insert(
            player.id,
            TranslationRotation {
                translation: transform.translation,
                rotation: transform.rotation,
            },
        );
    }

    let sync_message = bincode::serialize(&players).unwrap();
    server.broadcast_message(DefaultChannel::Unreliable, sync_message);
}

fn update_players_system(
    mut query: Query<(&mut ExternalForce, &Transform, &PlayerInput)>,
    time: Res<Time>,
    mut commands: Commands,
    mut server: ResMut<RenetServer>,
) {
    for (mut rigidbody, transform, input) in query.iter_mut() {
        let rotation = (input.rotate_right as i8 - input.rotate_left as i8) as f32;
        let thrust_longitudal = (input.thrust_reverse as i8 - input.thrust_forward as i8) as f32;
        let thrust_lateral = (input.thrust_left as i8 - input.thrust_right as i8) as f32;
        let thrust_vertical = (input.thrust_down as i8 - input.thrust_up as i8) as f32;

        let longitudal_force = thrust_longitudal * PLAYER_MOVE_SPEED * 20.0 * transform.forward();
        let lateral_force = thrust_lateral * PLAYER_MOVE_SPEED * 5.0 * transform.right();
        let vertical_force = thrust_vertical * PLAYER_MOVE_SPEED * 10.0 * transform.down();

        rigidbody.force = longitudal_force + lateral_force + vertical_force;
        rigidbody.torque = rotation * transform.down() * PLAYER_MOVE_SPEED * 2.0;
        // transform.translation.x += x * PLAYER_MOVE_SPEED * time.delta_seconds();
        // transform.translation.z += y * PLAYER_MOVE_SPEED * time.delta_seconds();
    }
}
