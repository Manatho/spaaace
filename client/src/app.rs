use std::{net::UdpSocket, time::SystemTime};

use bevy::{
    app::App,
    math::vec3,
    prelude::{
        default, shape, Assets, Camera3dBundle, Color, Commands, Input, IntoSystemDescriptor,
        KeyCode, Mesh, PbrBundle, Res, ResMut, StandardMaterial, Transform, Vec3,
    },
    utils::HashMap,
    DefaultPlugins,
};

use bevy_inspector_egui::WorldInspectorPlugin;

use bevy_renet::{
    renet::{ClientAuthentication, DefaultChannel, RenetClient, RenetConnectionConfig},
    run_if_client_connected, RenetClientPlugin,
};
use spaaaace_shared::{Lobby, PlayerInput, ServerMessages, PROTOCOL_ID};

pub fn run() {
    App::default()
        // Plugins
        .add_plugins(DefaultPlugins)
        .insert_resource(Lobby::default())
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(RenetClientPlugin::default())
        .add_startup_system(spawn_stuff)
        .insert_resource(new_renet_client())
        .insert_resource(PlayerInput::default())
        .add_system(player_input)
        .add_system(client_send_input.with_run_criteria(run_if_client_connected))
        .add_system(client_sync_players.with_run_criteria(run_if_client_connected))
        // Run App
        .run();
}

fn spawn_stuff(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 6., 12.0).looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
        ..default()
    });
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

fn player_input(k_input: Res<Input<KeyCode>>, mut player_input: ResMut<PlayerInput>) {
    player_input.left = k_input.pressed(KeyCode::A);
    player_input.right = k_input.pressed(KeyCode::D);
    player_input.up = k_input.pressed(KeyCode::W);
    player_input.down = k_input.pressed(KeyCode::S);
    player_input.primary_fire = k_input.pressed(KeyCode::Space);
}

fn client_send_input(player_input: Res<PlayerInput>, mut client: ResMut<RenetClient>) {
    let input_message = bincode::serialize(&*player_input).unwrap();

    client.send_message(DefaultChannel::Reliable, input_message);
}

fn client_sync_players(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut client: ResMut<RenetClient>,
    mut lobby: ResMut<Lobby>,
) {
    while let Some(message) = client.receive_message(DefaultChannel::Reliable) {
        let server_message = bincode::deserialize(&message).unwrap();
        match server_message {
            ServerMessages::PlayerConnected { id } => {
                println!("Player {} connected.", id);
                let player_entity = commands
                    .spawn(PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                        transform: Transform::from_xyz(0.0, 0.5, 0.0),
                        ..Default::default()
                    })
                    .id();

                lobby.players.insert(id, player_entity);
            }
            ServerMessages::PlayerDisconnected { id } => {
                println!("Player {} disconnected.", id);
                if let Some(player_entity) = lobby.players.remove(&id) {
                    commands.entity(player_entity).despawn();
                }
            }
            ServerMessages::BulletSpawned { position, rotation } => {
                commands.spawn(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Icosphere {
                        radius: 0.2,
                        ..Default::default()
                    })),
                    material: materials.add(Color::rgb(1.0, 0.2, 0.2).into()),
                    transform: Transform {
                        translation: position,
                        rotation: rotation,
                        ..Default::default()
                    },
                    ..Default::default()
                });
            }
        }
    }

    while let Some(message) = client.receive_message(DefaultChannel::Unreliable) {
        let players: HashMap<u64, [f32; 3]> = bincode::deserialize(&message).unwrap();
        for (player_id, translation) in players.iter() {
            if let Some(player_entity) = lobby.players.get(player_id) {
                let transform = Transform {
                    translation: (*translation).into(),
                    ..Default::default()
                };
                commands.entity(*player_entity).insert(transform);
            }
        }
    }
}
