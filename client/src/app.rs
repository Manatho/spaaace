use std::f32::consts::PI;

use app::{
    asteroid::AsteroidPlugin,
    camera::{OrbitCamera, OrbitCameraPlugin},
    capture_point::ClientCapturePointPlugin,
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
    prelude::{
        default, AmbientLight, Camera, Camera3dBundle, ClearColor, Color, Commands,
        DirectionalLight, DirectionalLightBundle, PluginGroup, Quat, ResMut, Transform, Vec3,
    },
    window::{WindowDescriptor, WindowPlugin},
    DefaultPlugins,
};

use bevy_egui::EguiPlugin;
use bevy_hanabi::HanabiPlugin;

use bevy_mod_gizmos::GizmosPlugin;

use bevy_rapier3d::prelude::{NoUserData, RapierPhysicsPlugin, RapierDebugRenderPlugin};
use spaaaace_shared::{Lobby, ServerMessages};
use bevy_rapier3d::prelude::{RapierPhysicsPlugin, NoUserData};
use spaaaace_shared::{weapons::WeaponsPlugin, Lobby, NetworkContext, ServerMessages};

use crate::networking::ClientNetworkingPlugin;

pub fn run() {
    App::default()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "Spaaace Client".to_string(),
                width: 1280.,
                height: 640.,
                ..default()
            },
            ..default()
        }))
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
        .add_system(lerp_transform_targets)
        .add_system(handle_ship_model_load)
        .add_system(handle_turret_model_load)
        .add_plugin(OrbitCameraPlugin)
        // ------------------
        // UI Stuff
        // ------------------
        .insert_resource(ClientGameState { is_paused: false })
        .add_plugin(GameUIPlugin)
        // ------------------
        // Gameplay stuff
        // ------------------
        .insert_resource(NetworkContext { is_server: false })
        .add_plugin(ClientNetworkingPlugin)
        .insert_resource(Lobby::default())
        .add_plugin(ClientPlayerPlugin {})
        .add_plugin(WeaponsPlugin {})
        .add_plugin(ClientCapturePointPlugin {})
        .add_plugin(AsteroidPlugin)
        .add_event::<ServerMessages>()
        // ------------------
        // Debug
        // ------------------
        //.add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(EguiPlugin)
        .add_plugin(GizmosPlugin)
        .add_plugin(CubemapPlugin)
        .insert_resource(ClearColor(Color::rgb(0.01, 0.01, 0.01))) // Used by guis
        .add_system(fps_gui)
        .add_system(team_swap_gui)
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
