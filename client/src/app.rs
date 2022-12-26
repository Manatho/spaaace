use std::{f32::consts::PI, net::UdpSocket, time::SystemTime};

use app::{
    particles::ThrusterModifier,
    utils::{lerp_transform_targets, LerpTransformTarget},
};
use bevy::{
    app::App,
    core_pipeline::{
        bloom::BloomSettings,
        fxaa::{Fxaa, FxaaPlugin},
    },
    gltf::{Gltf, GltfNode},
    input::mouse::{MouseMotion, MouseWheel},
    prelude::{
        default, shape, AmbientLight, AssetServer, Assets, BuildChildren, Camera, Camera3dBundle,
        ClearColor, Color, Commands, Component, DirectionalLight, DirectionalLightBundle, Entity,
        EventReader, Handle, Input, IntoSystemDescriptor, KeyCode, Mesh, Msaa, PbrBundle,
        PluginGroup, Quat, Query, Res, ResMut, SpatialBundle, StandardMaterial, Transform, Vec2,
        Vec3, Vec4, With, Without,
    },
    scene::SceneBundle,
    time::Time,
    utils::HashMap,
    window::{PresentMode, WindowDescriptor, WindowPlugin, Windows},
    DefaultPlugins,
};

use bevy_hanabi::{
    BillboardModifier, ColorOverLifetimeModifier, EffectAsset, Gradient, HanabiPlugin,
    ParticleEffectBundle, ParticleLifetimeModifier, PositionSphereModifier, ShapeDimension,
    SizeOverLifetimeModifier, Spawner,
};
use bevy_inspector_egui::WorldInspectorPlugin;

use bevy_renet::{
    renet::{ClientAuthentication, DefaultChannel, RenetClient, RenetConnectionConfig},
    run_if_client_connected, RenetClientPlugin,
};
use rand::Rng;
use spaaaace_shared::{
    Lobby, PlayerInput, ServerMessages, TranslationRotation, PROTOCOL_ID, SERVER_TICKRATE,
};

#[derive(Component)]
struct LocalPlayer;

pub fn run() {
    App::default()
        // Plugins
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "Spaaace Client".to_string(),
                width: 640.,
                height: 320.,
                present_mode: PresentMode::AutoVsync,
                ..default()
            },
            ..default()
        }))
        .add_plugin(HanabiPlugin)
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
        .add_system(lerp_transform_targets)
        .add_system(spawn_gltf_objects)
        .insert_resource(ClearColor(Color::rgb(0.01, 0.01, 0.01)))
        // Run App
        .run();
}

fn init(mut commands: Commands, mut ambient_light: ResMut<AmbientLight>, ass: Res<AssetServer>) {
    commands
        .spawn(Camera3dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },

            transform: Transform::from_xyz(0.0, 6., -12.0)
                .looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
            ..default()
        })
        .insert(OrbitCamera { zoom: 50.0 })
        .insert(BloomSettings { ..default() })
        .insert(Fxaa { ..default() });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10000.0,
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
    ambient_light.brightness = 0.01;

    let mut rng = rand::thread_rng();
    for i in 0..100 {
        commands.spawn(SceneBundle {
            scene: ass.load("asteroid.glb#Scene0"),
            transform: Transform::from_translation(Vec3 {
                x: rng.gen::<f32>() * 250.0,
                y: rng.gen::<f32>() * 250.0,
                z: rng.gen::<f32>() * 250.0,
            }),
            ..default()
        });
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
    mut client: ResMut<RenetClient>,
    mut lobby: ResMut<Lobby>,
    ass: Res<AssetServer>,
) {
    while let Some(message) = client.receive_message(DefaultChannel::Reliable) {
        let server_message = bincode::deserialize(&message).unwrap();
        match server_message {
            ServerMessages::PlayerConnected { id } => {
                println!("Player {} connected.", id);

                let my_gltf = ass.load("test_ship.glb");
                let mut cmd =
                    commands.spawn((SpatialBundle { ..default() }, ShipModelLoadHandle(my_gltf)));

                if id == client.client_id() {
                    cmd.insert(LocalPlayer {});
                }

                let player_entity = cmd.id();

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

fn camera_follow_local_player(
    mut camera_query: Query<(&mut Transform, &mut OrbitCamera), Without<LocalPlayer>>,
    local_player_query: Query<&Transform, With<LocalPlayer>>,
    mut motion_evr: EventReader<MouseMotion>,
    mut scroll_evr: EventReader<MouseWheel>,
    windows: Res<Windows>,
) {
    let mut rotation_move = Vec2::ZERO;

    for ev in motion_evr.iter() {
        rotation_move += ev.delta * 10.0;
    }

    let scroll_zoom = scroll_evr.iter().map(|x| x.y).sum::<f32>();

    for (mut transform, mut orbit_camera) in camera_query.iter_mut() {
        orbit_camera.zoom += scroll_zoom;
        match local_player_query.get_single() {
            Ok(local_player_transform) => {
                if rotation_move.length_squared() > 0.0 {
                    let window = get_primary_window_size(&windows);
                    let delta = (rotation_move / window) / PI;
                    let yaw = Quat::from_rotation_y(-delta.x);
                    let pitch = Quat::from_rotation_x(-delta.y);
                    transform.rotation = yaw * transform.rotation; // rotate around global y axis
                    transform.rotation = transform.rotation * pitch; // rotate around local x axis
                }
                transform.translation =
                    local_player_transform.translation + transform.back() * orbit_camera.zoom;
                // transform.rotation *= Rot
            }
            Err(_) => {}
        }
    }
}

fn get_primary_window_size(windows: &Res<Windows>) -> Vec2 {
    let window = windows.get_primary().unwrap();
    let window = Vec2::new(window.width() as f32, window.height() as f32);
    window
}

#[derive(Component)]
pub struct OrbitCamera {
    pub zoom: f32,
}

#[derive(Component)]
struct ShipModelLoadHandle(Handle<Gltf>);

fn spawn_gltf_objects(
    mut commands: Commands,
    query: Query<(Entity, &ShipModelLoadHandle)>,
    assets_gltf: Res<Assets<Gltf>>,
    assets_gltfnode: Res<Assets<GltfNode>>,
    mut effects: ResMut<Assets<EffectAsset>>,
) {
    for (entity, handle) in query.iter() {
        if let Some(gltf) = assets_gltf.get(&handle.0) {
            let mut gradient = Gradient::new();
            gradient.add_key(0.0, Vec4::new(0.0, 1.0, 1.0, 1.0) * 3.0);
            gradient.add_key(1.0, Vec4::new(0.0, 1.0, 1.0, 0.0));

            println!("TEST");
            let spawner = Spawner::rate(100.0.into());
            let effect = effects.add(
                EffectAsset {
                    name: "Impact".into(),
                    capacity: 32768,
                    spawner,
                    ..Default::default()
                }
                .init(ParticleLifetimeModifier { lifetime: 1.0 })
                .init(PositionSphereModifier {
                    radius: 0.75,
                    speed: 0.0.into(),
                    dimension: ShapeDimension::Volume,
                    ..Default::default()
                })
                .render(SizeOverLifetimeModifier {
                    gradient: Gradient::constant(Vec2::splat(0.05)),
                })
                .render(ColorOverLifetimeModifier { gradient })
                .render(BillboardModifier {}),
            );

            println!("Loaded GLTF, spawning...");
            // spawn the first scene in the file
            let model = commands
                .spawn(SceneBundle {
                    scene: gltf.scenes[0].clone(),
                    ..Default::default()
                })
                .id();

            // for node_handle in gltf.nodes.iter() {
            //     if let Some(node) = assets_gltfnode.get(node_handle) {
            //         node.
            //     }
            // }

            let mut thruster_points: Vec<Entity> = vec![];

            for node_name in gltf.named_nodes.keys().into_iter() {
                println!("NODE NAME: {}", node_name);
                if node_name.contains("forward_thrusters") {
                    if let Some(node) = assets_gltfnode.get(&gltf.named_nodes[node_name]) {
                        let thruster = commands
                            .spawn(ParticleEffectBundle::new(effect.clone()))
                            .insert(node.transform)
                            .id();
                        thruster_points.push(thruster);
                    }
                }
            }

            commands
                .entity(entity)
                .push_children(&[model])
                .push_children(&thruster_points)
                .remove::<ShipModelLoadHandle>();

            // spawn the scene named "YellowCar"
            // commands.spawn(SceneBundle {
            //     scene: gltf.named_scenes["YellowCar"].clone(),
            //     transform: Transform::from_xyz(1.0, 2.0, 3.0),
            //     ..Default::default()
            // });

            // PERF: the `.clone()`s are just for asset handles, don't worry :)
        }
    }
}
