use std::{f32::consts::PI, net::UdpSocket, time::SystemTime};

use app::{
    camera::{OrbitCamera, OrbitCameraPlugin, OrbitCameraTarget},
    capture_point::capture_point::ForceFieldMaterial,
    controls::player_input,
    debug::fps::{fps_gui, team_swap_gui},
    ui::GameUIPlugin,
    utils::{lerp_transform_targets, LerpTransformTarget},
};
use bevy::{
    app::App,
    core_pipeline::{bloom::BloomSettings, fxaa::Fxaa},
    diagnostic::FrameTimeDiagnosticsPlugin,
    gltf::{Gltf, GltfNode},
    pbr::NotShadowCaster,
    prelude::{
        default, shape, AmbientLight, AssetServer, Assets, BuildChildren, Camera, Camera3dBundle,
        ClearColor, Color, Commands, Component, DirectionalLight, DirectionalLightBundle, Entity,
        Handle, IntoSystemDescriptor, MaterialMeshBundle, MaterialPlugin, Mesh, PbrBundle,
        PluginGroup, Quat, Query, Res, ResMut, SpatialBundle, StandardMaterial, Transform, Vec2,
        Vec3, Vec4,
    },
    scene::SceneBundle,
    time::Time,
    utils::HashMap,
    window::{WindowDescriptor, WindowPlugin},
    DefaultPlugins,
};

use bevy_egui::EguiPlugin;
use bevy_hanabi::{
    BillboardModifier, ColorOverLifetimeModifier, EffectAsset, Gradient, HanabiPlugin,
    ParticleEffectBundle, ParticleLifetimeModifier, PositionSphereModifier, ShapeDimension,
    SizeOverLifetimeModifier, Spawner,
};

use bevy_renet::{
    renet::{ClientAuthentication, DefaultChannel, RenetClient, RenetConnectionConfig},
    run_if_client_connected, RenetClientPlugin,
};
use rand::Rng;
use spaaaace_shared::{
    team::team_enum::Team, util::Random, ClientMessages, Lobby, PlayerInput, ServerMessages,
    TranslationRotation, PROTOCOL_ID, SERVER_TICKRATE,
};

pub fn run() {
    App::default()
        // Plugins
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "Spaaace Client".to_string(),
                width: 640.,
                height: 320.,
                ..default()
            },
            ..default()
        }))
        .add_plugin(HanabiPlugin)
        .insert_resource(Lobby::default())
        //.add_plugin(WorldInspectorPlugin::new())
        .add_plugin(EguiPlugin)
        .add_plugin(RenetClientPlugin::default())
        .add_startup_system(init)
        .insert_resource(new_renet_client())
        .insert_resource(PlayerInput::default())
        .add_system(player_input)
        .add_system(client_send_input.with_run_criteria(run_if_client_connected))
        .add_system(client_sync_players.with_run_criteria(run_if_client_connected))
        .add_system(lerp_transform_targets)
        .add_plugin(MaterialPlugin::<ForceFieldMaterial>::default())
        .add_system(spawn_gltf_objects)
        .insert_resource(ClearColor(Color::rgb(0.01, 0.01, 0.01)))
        .add_system(fps_gui)
        .add_system(team_swap_gui)
        // Run App
        .add_plugin(OrbitCameraPlugin)
        .add_plugin(GameUIPlugin)
        .add_system(move_bullet)
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
    for _ in 0..100 {
        commands.spawn(SceneBundle {
            scene: ass.load("asteroid.glb#Scene0"),
            transform: Transform {
                translation: Vec3 {
                    x: rng.gen::<f32>() * 250.0,
                    y: rng.gen::<f32>() * 250.0,
                    z: rng.gen::<f32>() * 250.0,
                },
                scale: Vec3::splat(2.0 + rng.gen::<f32>() * 8.0),
                rotation: Quat::random(),
                ..default()
            },
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

fn client_send_input(player_input: Res<PlayerInput>, mut client: ResMut<RenetClient>) {
    let client_message = ClientMessages::PlayerInput {
        input: *player_input,
    };
    let input_message = bincode::serialize(&client_message).unwrap();
    client.send_message(DefaultChannel::Reliable, input_message);
}

#[derive(Component)]
struct Bullet {}

fn move_bullet(mut query: Query<(&Bullet, &mut Transform)>, time: Res<Time>) {
    for (_, mut transform) in query.iter_mut() {
        let dir = transform.forward();
        transform.translation += dir * time.delta_seconds() * 20.;
    }
}

fn client_sync_players(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut force_field_materials: ResMut<Assets<ForceFieldMaterial>>,
    mut client: ResMut<RenetClient>,
    mut lobby: ResMut<Lobby>,
    query: Query<&Handle<ForceFieldMaterial>>,
    ass: Res<AssetServer>,
    time: Res<Time>,
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
                    cmd.insert(OrbitCameraTarget {});
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
                commands
                    .spawn(PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Capsule {
                            depth: 0.5,
                            radius: 0.01,
                            ..Default::default()
                        })),
                        material: materials.add(StandardMaterial {
                            base_color: Color::BLACK,
                            perceptual_roughness: 1.,
                            emissive: Color::rgb(1., 0.2, 0.2) * 5.,
                            ..default()
                        }),
                        transform: Transform {
                            translation: position,
                            rotation: rotation,
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .insert(Bullet {});
            }
            ServerMessages::CapturePointSpawned {
                position,
                rotation,
                id,
                owner,
                progress: _,
            } => {
                let capture_entity = commands
                    .spawn(MaterialMeshBundle {
                        mesh: meshes.add(
                            shape::Icosphere {
                                radius: 50.,
                                subdivisions: 8,
                            }
                            .into(),
                        ),
                        material: force_field_materials.add(ForceFieldMaterial {
                            color: match owner {
                                Team::Neutral => Color::WHITE,
                                Team::Red => Color::RED,
                                Team::Blue => Color::BLUE,
                            },
                            prev_color: Color::WHITE,
                            last_color_change: time.elapsed_seconds(),
                        }),
                        transform: Transform {
                            rotation: rotation,
                            translation: position,
                            ..Default::default()
                        },
                        ..default()
                    })
                    .insert(NotShadowCaster)
                    .id();

                lobby.capture_points.insert(id, capture_entity);
            }
            ServerMessages::CapturePointUpdate {
                id,
                owner,
                attacker: _,
                progress: _,
            } => {
                if let Some(entity) = lobby.capture_points.get(&id) {
                    match query.get(*entity) {
                        Ok(material) => {
                            if let Some(material) = force_field_materials.get_mut(material) {
                                let next_color = match owner {
                                    Team::Neutral => Color::WHITE,
                                    Team::Red => Color::RED,
                                    Team::Blue => Color::BLUE,
                                };
                                if material.color != next_color {
                                    material.prev_color = material.color;
                                    material.color = next_color;
                                    material.last_color_change = time.elapsed_seconds();

                                    println!(
                                        "{:?} {:?} {:?}",
                                        material.color,
                                        material.prev_color,
                                        material.last_color_change
                                    );
                                }
                            }
                        }
                        _ => (),
                    }
                }
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
            let mut thruster_points: Vec<Entity> = vec![];

            for node_name in gltf.named_nodes.keys().into_iter() {
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
        }
    }
}
