use std::{net::UdpSocket, time::SystemTime};

use bevy::{
    prelude::{
        default, info, App, Camera3d, Camera3dBundle, Commands, EventWriter, PluginGroup,
        PointLight, PointLightBundle, Query, ResMut, Transform, Vec3, With, Without,
    },
    window::{PresentMode, Window, WindowPlugin, WindowResolution},
    DefaultPlugins,
};

use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_mod_gizmos::GizmosPlugin;
use bevy_rapier3d::{
    prelude::{NoUserData, RapierPhysicsPlugin},
    render::RapierDebugRenderPlugin,
};
use bevy_renet::{
    renet::{
        DefaultChannel, RenetConnectionConfig, RenetServer, ServerAuthentication, ServerConfig,
    },
    RenetServerPlugin,
};

use spaaaace_shared::{
    asteroid::AsteroidPlugin, cooldown::CooldownPlugin, health::HealthPlugin, player::Player,
    weapons::WeaponsPlugin, ClientMessages, Lobby, NetworkContext, NetworkIdProvider, PROTOCOL_ID,
};

use crate::{capture_point::CapturePointPlugin, player::PlayerPlugin};

pub mod capture_point;
pub mod player;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
struct FixedUpdateStage;

fn main() {
    info!("Naia Bevy Server Demo starting up");

    // Build App
    App::default()
        // Plugins
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Spaaace Server".to_string(),
                resolution: WindowResolution::new(1280., 720.),
                present_mode: PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }))
        .add_startup_system(setup)
        // ------------------
        // Third party
        // ------------------
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        // ------------------
        // Networking stuff
        // ------------------
        .insert_resource(Lobby::default())
        .insert_resource(NetworkIdProvider::new())
        .insert_resource(NetworkContext { is_server: true })
        .add_event::<ClientEvent>()
        .add_plugin(RenetServerPlugin::default())
        .insert_resource(new_renet_server())
        .add_system(server_update_system)
        // ------------------
        // Gameplay stuff
        // ------------------
        .add_plugin(HealthPlugin)
        .add_plugin(WeaponsPlugin {})
        .add_plugin(AsteroidPlugin {})
        .add_plugin(PlayerPlugin)
        .add_plugin(CapturePointPlugin)
        .add_plugin(CooldownPlugin)
        // ------------------
        // Debugging stuff
        // ------------------
        .add_plugin(GizmosPlugin)
        .add_plugin(WorldInspectorPlugin::default())
        // .add_plugin(InputPlugin::default())
        // .add_plugin(ScenePlugin::default())
        // .add_plugin(WindowPlugin::default())
        // .add_plugin(WinitPlugin::default())
        // .add_plugin(RenderPlugin::default())
        .add_system(camera_follow_players)
        .run();
}

fn setup(mut commands: Commands) {
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
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

#[derive(Clone)]
struct ClientEvent {
    pub message: ClientMessages,
    pub client_id: u64,
}

fn server_update_system(
    mut server: ResMut<RenetServer>,
    mut client_message_event_writer: EventWriter<ClientEvent>,
) {
    for client_id in server.clients_id().into_iter() {
        while let Some(message) = server.receive_message(client_id, DefaultChannel::Reliable) {
            let message: ClientMessages = bincode::deserialize(&message).unwrap();
            client_message_event_writer.send(ClientEvent { message, client_id });
        }
    }
}

fn camera_follow_players(
    mut query_cam: Query<&mut Transform, (With<Camera3d>, Without<Player>)>,
    query_players: Query<&Transform, With<Player>>,
) {
    if query_players.is_empty() {
        return;
    }
    let mut player_count = 0;
    let mut avg = Vec3::ZERO;
    for player_transform in query_players.iter() {
        avg += player_transform.translation;
        player_count += 1;
    }

    avg /= Vec3::splat(player_count as f32);

    let mut max_dist_from_avg: f32 = 0.0;

    for player_transform in query_players.iter() {
        let dist = player_transform.translation.distance(avg);
        max_dist_from_avg = max_dist_from_avg.max(dist);
    }

    for mut transform in query_cam.iter_mut() {
        transform.look_at(avg, Vec3::Y);

        let back = transform.back();
        transform.translation = avg + back * (max_dist_from_avg * 3.0 + 80.);
    }
}
