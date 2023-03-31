use std::f32::consts::PI;

use app::{
    camera::{OrbitCamera, OrbitCameraPlugin},
    capture_point::ClientCapturePointPlugin,
    controls::ControlsPlugin,
    debug::fps::{fps_gui, team_swap_gui},
    game_state::ClientGameState,
    player::ClientPlayerPlugin,
    skybox::cubemap::CubemapPlugin,
    ui::GameUIPlugin,
    utils::{handle_ship_model_load, handle_turret_model_load, lerp_transform_targets},
};
use bevy::{
    app::App,
    core_pipeline::{bloom::BloomSettings, fxaa::Fxaa},
    diagnostic::FrameTimeDiagnosticsPlugin,
    math::vec3,
    pbr::NotShadowCaster,
    prelude::{
        default, shape, AmbientLight, AssetPlugin, AssetServer, Assets, Camera, Camera3dBundle,
        ClearColor, Color, Commands, DirectionalLight, DirectionalLightBundle, FogFalloff,
        FogSettings, Mesh, PbrBundle, PluginGroup, Quat, Res, ResMut, StandardMaterial, Transform,
        Vec3,
    },
    window::{Window, WindowPlugin, WindowResolution},
    DefaultPlugins,
};

use bevy_egui::EguiPlugin;
use bevy_hanabi::HanabiPlugin;
use bevy_mod_gizmos::GizmosPlugin;

use bevy_rapier3d::prelude::{NoUserData, RapierDebugRenderPlugin, RapierPhysicsPlugin};
use bevy_scene_hook::HookPlugin;
use spaaaace_shared::{
    asteroid::AsteroidPlugin, weapons::WeaponsPlugin, Lobby, NetworkContext, ServerMessages,
};

use crate::networking::ClientNetworkingPlugin;

pub fn run() {
    App::default()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Spaaace Client".to_string(),
                        resolution: WindowResolution::new(1280., 640.),
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    watch_for_changes: true,
                    ..Default::default()
                }),
        )
        .add_startup_system(init)
        // ------------------
        // Effects
        // ------------------
        .add_plugin(HanabiPlugin)
        // ------------------
        // Third party
        // ------------------
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        // ------------------
        // Utils
        // ------------------
        .add_plugin(HookPlugin)
        .add_system(lerp_transform_targets)
        .add_system(handle_ship_model_load)
        .add_system(handle_turret_model_load)
        .add_plugin(OrbitCameraPlugin)
        // ------------------
        // UI Stuff
        // ------------------
        .insert_resource(ClientGameState {
            is_paused: false,
            is_focused: true,
        })
        .add_plugin(GameUIPlugin)
        // ------------------
        // Gameplay stuff
        // ------------------
        .insert_resource(NetworkContext { is_server: false })
        .add_plugin(ClientNetworkingPlugin)
        .insert_resource(Lobby::default())
        .add_plugin(ClientPlayerPlugin {})
        .add_plugin(ControlsPlugin {})
        .add_plugin(WeaponsPlugin {})
        .add_plugin(ClientCapturePointPlugin {})
        .add_plugin(AsteroidPlugin)
        .add_event::<ServerMessages>()
        // ------------------
        // Debug
        // ------------------
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        //.add_plugin(WorldInspectorPlugin::default())
        .add_plugin(EguiPlugin)
        .add_plugin(GizmosPlugin)
        .add_plugin(CubemapPlugin)
        .insert_resource(ClearColor(Color::rgb(0.01, 0.01, 0.01))) // Used by guis
        .add_system(fps_gui)
        .add_system(team_swap_gui)
        .run();
}

fn init(
    mut commands: Commands,
    mut ambient_light: ResMut<AmbientLight>,
    _asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
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
        .insert(FogSettings {
            color: Color::rgba(0.1, 0.2, 0.4, 1.0) * 0.04,
            directional_light_color: Color::rgba(1.0, 0.95, 0.75, 0.5),
            directional_light_exponent: 30.0,
            falloff: FogFalloff::from_visibility_colors(
                3000.0, // distance in world units up to which objects retain visibility (>= 5% contrast)
                Color::rgb(0.35, 0.5, 0.66), // atmospheric extinction color (after light is lost due to absorption by atmospheric particles)
                Color::rgb(0.8, 0.844, 1.0), // atmospheric inscattering color (light gained due to scattering from the sun)
            ),
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

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere::default())),
            material: materials.add(StandardMaterial {
                base_color: Color::hex("888888").unwrap(),
                unlit: true,
                cull_mode: None,
                ..default()
            }),
            transform: Transform::from_scale(Vec3::splat(10000.0)),
            ..default()
        },
        NotShadowCaster,
    ));

    // commands.spawn(EnvironmentMapLight {
    //     diffuse_map: asset_server.load("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2"),
    //     specular_map: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
    // });
}
