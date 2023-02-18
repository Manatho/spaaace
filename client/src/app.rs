use std::{f32::consts::PI, net::UdpSocket, time::SystemTime};

use app::{
    camera::{OrbitCamera, OrbitCameraPlugin, OrbitCameraTarget},
    capture_point::ClientCapturePointPlugin,
    controls::player_input,
    debug::fps::{fps_gui, team_swap_gui},
    game_state::ClientGameState,
    player::{ClientPlayerPlugin, ShipModelLoadHandle},
    skybox::cubemap::CubemapPlugin,
    ui::GameUIPlugin,
    utils::{lerp_transform_targets, LerpTransformTarget},
    weapons::ClientWeaponsPlugin,
};
use bevy::{
    app::App,
    core_pipeline::{bloom::BloomSettings, fxaa::Fxaa},
    diagnostic::FrameTimeDiagnosticsPlugin,
    gltf::{Gltf, GltfNode},
    math::vec3,
    prelude::{
        default, AmbientLight, AssetServer, Assets, BuildChildren, Camera, Camera3dBundle,
        ClearColor, Color, Commands, DirectionalLight, DirectionalLightBundle, Entity, EventWriter,
        IntoSystemDescriptor, PluginGroup, Quat, Query, Res, ResMut, Transform, Vec2, Vec3, Vec4,
    },
    scene::SceneBundle,
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

use bevy_mod_gizmos::GizmosPlugin;
use bevy_renet::{
    renet::{ClientAuthentication, DefaultChannel, RenetClient, RenetConnectionConfig},
    run_if_client_connected, RenetClientPlugin,
};

use spaaaace_shared::{
    player::player_input::PlayerInput, ClientMessages, Lobby, ServerMessages, TranslationRotation,
    PROTOCOL_ID, SERVER_TICKRATE,
};

pub fn run() {
    App::default()
        // Plugins
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "Spaaace Client".to_string(),
                width: 1280.,
                height: 640.,
                ..default()
            },
            ..default()
        }))
        .insert_resource(ClientGameState { is_paused: false })
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
        .add_system(client_reliable_message_handler.with_run_criteria(run_if_client_connected))
        .add_system(client_unreliable_message_handler.with_run_criteria(run_if_client_connected))
        .add_system(lerp_transform_targets)
        .add_system(spawn_gltf_objects)
        .insert_resource(ClearColor(Color::rgb(0.01, 0.01, 0.01)))
        .add_system(fps_gui)
        .add_system(team_swap_gui)
        // .add_system(client_update_system)
        .add_plugin(ClientPlayerPlugin {})
        .add_plugin(ClientWeaponsPlugin {})
        .add_plugin(ClientCapturePointPlugin {})
        .add_event::<ServerMessages>()
        // Run App
        .add_plugin(OrbitCameraPlugin)
        .add_plugin(GameUIPlugin)
        // Debug
        .add_plugin(GizmosPlugin)
        .add_plugin(CubemapPlugin)
        .run();
}

fn init(mut commands: Commands, mut ambient_light: ResMut<AmbientLight>) {
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
        .insert(OrbitCamera {
            zoom: 50.0,
            offset: vec3(0., 10., 0.),
        })
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

    ambient_light.color = Color::hsl(207.0, 0.5, 0.4);
    ambient_light.brightness = 0.7;
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
