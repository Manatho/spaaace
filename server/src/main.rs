use std::{net::UdpSocket, time::SystemTime};

use bevy::{
    prelude::{
        default, info, App, Camera3dBundle, Commands, EventWriter, PluginGroup, ResMut, StageLabel,
        Transform, Vec3,
    },
    window::{PresentMode, WindowDescriptor, WindowPlugin},
    DefaultPlugins,
};

use bevy_inspector_egui::WorldInspectorPlugin;
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

use player::Player;
use spaaaace_shared::{ClientMessages, Lobby, PROTOCOL_ID};

use crate::{capture_point::CapturePointPlugin, player::PlayerPlugin, weapons::WeaponsPlugin};

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
        .add_event::<ClientEvent>()
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
