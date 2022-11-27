mod capture_point;
mod client;
mod networking;
mod player;
mod server;
mod server_main;
mod ship;
mod team;
use client::events as ClientEvents;
use client::init::client_init;
use naia_bevy_client::ClientConfig;
use naia_bevy_client::{Plugin as ClientPlugin, Stage as ClientStage};
use naia_bevy_server::{Plugin as ServerPlugin, ServerConfig, Stage as ServerStage};
use server::{events as ServerEvents, server_init, tick};

use bevy::{
    pbr::NotShadowCaster,
    prelude::{
        default, shape, App, AssetPlugin, Assets, Camera3dBundle, Color, Commands,
        DirectionalLight, DirectionalLightBundle, IntoSystemDescriptor, MaterialMeshBundle, Mesh,
        OrthographicProjection, PbrBundle, PluginGroup, Quat, ResMut, StandardMaterial, Transform,
        Vec3,
    },
    utils::HashSet,
    DefaultPlugins,
};
use capture_point::CapturePointPlugin;
use networking::{channels::Channels, protocol::Protocol, shared::shared_config};
use player::PlayerPlugin;
use ship::{space_ship::SpaceShip, SpaceShipPlugin};
use std::f32::consts::PI;

use crate::{
    capture_point::capture_point::{CaptureSphere, ForceFieldMaterial},
    player::player::Player,
    team::team_enum::Team,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            // Tell the asset server to watch for asset changes on disk:
            watch_for_changes: true,
            ..default()
        }))
        .add_plugin(SpaceShipPlugin)
        .add_plugin(CapturePointPlugin)
        .add_plugin(PlayerPlugin)
        .add_startup_system(setup)
        //naia stuff
        .add_plugin(ServerPlugin::<Protocol, Channels>::new(
            ServerConfig::default(),
            shared_config(),
        ))
        .add_plugin(ClientPlugin::<Protocol, Channels>::new(
            ClientConfig::default(),
            shared_config(),
        ))
        // Startup System
        .add_startup_system(server_init)
        .add_startup_system(client_init.after(server_init))
        .add_system_to_stage(ClientStage::ReceiveEvents, ClientEvents::spawn_entity_event)
        // Receive Server Events
        .add_system_to_stage(
            ServerStage::ReceiveEvents,
            ServerEvents::authorization_event,
        )
        .add_system_to_stage(ServerStage::ReceiveEvents, ServerEvents::connection_event)
        .add_system_to_stage(
            ServerStage::ReceiveEvents,
            ServerEvents::disconnection_event,
        )
        .add_system_to_stage(
            ServerStage::ReceiveEvents,
            ServerEvents::receive_message_event,
        )
        // Gameplay Loop on Tick
        .add_system_to_stage(ServerStage::Tick, tick)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut force_field_materials: ResMut<Assets<ForceFieldMaterial>>,
) {
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(shape::Cube { size: 1. }.into()),
            material: materials.add(Color::GREEN.into()),
            ..default()
        })
        .insert(SpaceShip { hp: 20 })
        .insert(Player { team: Team::Blue });

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
            ..default()
        })
        .insert(NotShadowCaster)
        .insert(CaptureSphere {
            radius: 3.,
            progress: 0.0,
            attackers: HashSet::new(),
            owner: Team::Neutral,
        });

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 6., 12.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
        ..default()
    });

    const HALF_SIZE: f32 = 10.0;
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            // Configure the projection to better fit the scene
            shadow_projection: OrthographicProjection {
                left: -HALF_SIZE,
                right: HALF_SIZE,
                bottom: -HALF_SIZE,
                top: HALF_SIZE,
                near: -10.0 * HALF_SIZE,
                far: 10.0 * HALF_SIZE,
                ..default()
            },
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
}
