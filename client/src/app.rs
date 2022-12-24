use std::{f32::consts::PI, net::UdpSocket, time::SystemTime};

use app::{
    capture_point::capture_point::ForceFieldMaterial,
    utils::{lerp_transform_targets, LerpTransformTarget},
};
use bevy::{
    app::App,
    core_pipeline::bloom::BloomSettings,
    gltf::Gltf,
    math::vec3,
    pbr::NotShadowCaster,
    prelude::{
        default, shape, AmbientLight, AssetServer, Assets, Camera, Camera3d, Camera3dBundle, Color,
        Commands, Component, DirectionalLight, DirectionalLightBundle, Handle, Input,
        IntoSystemDescriptor, KeyCode, MaterialMeshBundle, MaterialPlugin, Mesh, PbrBundle, Quat,
        Query, Res, ResMut, Resource, StandardMaterial, Transform, Vec3, With, Without,
    },
    scene::SceneBundle,
    time::Time,
    utils::HashMap,
    DefaultPlugins,
};

use bevy_inspector_egui::WorldInspectorPlugin;

use bevy_renet::{
    renet::{ClientAuthentication, DefaultChannel, RenetClient, RenetConnectionConfig},
    run_if_client_connected, RenetClientPlugin,
};
use spaaaace_shared::{
    Lobby, PlayerInput, ServerMessages, TranslationRotation, PROTOCOL_ID, SERVER_TICKRATE,
};

#[derive(Component)]
struct LocalPlayer;

pub fn run() {
    App::default()
        // Plugins
        .add_plugins(DefaultPlugins)
        .insert_resource(Lobby::default())
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(RenetClientPlugin::default())
        .add_startup_system(init)
        .insert_resource(new_renet_client())
        .insert_resource(PlayerInput::default())
        .add_system(player_input)
        .add_system(client_send_input.with_run_criteria(run_if_client_connected))
        .add_system(client_sync_players.with_run_criteria(run_if_client_connected))
        .add_system(camera_follow_local_player)
        .add_startup_system(load_gltf)
        .add_system(lerp_transform_targets)
        .add_plugin(MaterialPlugin::<ForceFieldMaterial>::default())
        // Run App
        .run();
}

fn init(mut commands: Commands, mut ambient_light: ResMut<AmbientLight>) {
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                hdr: true,

                ..default()
            },

            transform: Transform::from_xyz(0.0, 6., -12.0)
                .looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
            ..default()
        },
        BloomSettings::default(),
    ));

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
        ..default()
    });

    ambient_light.color = Color::hsl(180.0, 1.0, 1.0);
    ambient_light.brightness = 2.0;
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
    player_input.rotate_left = k_input.pressed(KeyCode::A);
    player_input.rotate_right = k_input.pressed(KeyCode::D);
    player_input.thrust_forward = k_input.pressed(KeyCode::W);
    player_input.thrust_reverse = k_input.pressed(KeyCode::S);
    player_input.thrust_left = k_input.pressed(KeyCode::Q);
    player_input.thrust_right = k_input.pressed(KeyCode::E);
    player_input.thrust_up = k_input.pressed(KeyCode::Space);
    player_input.thrust_down = k_input.pressed(KeyCode::LControl);
    player_input.primary_fire = k_input.pressed(KeyCode::Return);
}

fn client_send_input(player_input: Res<PlayerInput>, mut client: ResMut<RenetClient>) {
    let input_message = bincode::serialize(&*player_input).unwrap();

    client.send_message(DefaultChannel::Reliable, input_message);
}

fn client_sync_players(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut force_field_materials: ResMut<Assets<ForceFieldMaterial>>,
    mut client: ResMut<RenetClient>,
    mut lobby: ResMut<Lobby>,
    my: Res<MyAssetPack>,
    assets_gltf: Res<Assets<Gltf>>,
) {
    while let Some(message) = client.receive_message(DefaultChannel::Reliable) {
        let server_message = bincode::deserialize(&message).unwrap();
        match server_message {
            ServerMessages::PlayerConnected { id } => {
                println!("Player {} connected.", id);
                if let Some(gltf) = assets_gltf.get(&my.0) {
                    let mut cmd = commands.spawn(SceneBundle {
                        scene: gltf.scenes[0].clone(),
                        ..Default::default()
                    });
                    println!(
                        "{} vs {} = {}",
                        id,
                        client.client_id(),
                        id == client.client_id()
                    );
                    if id == client.client_id() {
                        cmd.insert(LocalPlayer {});
                    }

                    let player_entity = cmd.id();

                    lobby.players.insert(id, player_entity);
                }
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
            ServerMessages::CapturePointSpawned { position, rotation } => {
                commands
                    .spawn(MaterialMeshBundle {
                        mesh: meshes.add(
                            shape::Icosphere {
                                radius: 3.,
                                subdivisions: 8,
                            }
                            .into(),
                        ),
                        material: force_field_materials.add(ForceFieldMaterial {}),
                        transform: Transform {
                            rotation: rotation,
                            translation: position,
                            ..Default::default()
                        },
                        ..default()
                    })
                    .insert(NotShadowCaster);
            }
        }
    }

    while let Some(message) = client.receive_message(DefaultChannel::Unreliable) {
        let players: HashMap<u64, TranslationRotation> = bincode::deserialize(&message).unwrap();
        for (player_id, translation_rotation) in players.iter() {
            if let Some(player_entity) = lobby.players.get(player_id) {
                commands.entity(*player_entity).insert(LerpTransformTarget {
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

/// Helper resource for tracking our asset
#[derive(Resource)]
struct MyAssetPack(Handle<Gltf>);

fn load_gltf(mut commands: Commands, ass: Res<AssetServer>) {
    let gltf = ass.load("test_ship.glb");
    commands.insert_resource(MyAssetPack(gltf));
}

fn camera_follow_local_player(
    mut camera_query: Query<(&mut Transform, &Camera3d), Without<LocalPlayer>>,
    local_player_query: Query<&Transform, With<LocalPlayer>>,
    time: Res<Time>,
) {
    for (mut transform, _) in camera_query.iter_mut() {
        match local_player_query.get_single() {
            Ok(local_player_transform) => {
                transform.translation = transform.translation.lerp(
                    local_player_transform.translation + vec3(0.0, 20.0, -50.0),
                    time.delta_seconds() * 1.,
                );
            }
            Err(_) => {}
        }
    }
}
