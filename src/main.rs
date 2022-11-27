mod capture_point;
mod player;
mod ship;
mod team;

use std::f32::consts::PI;

use bevy::{
    pbr::NotShadowCaster,
    prelude::{
        default, shape, App, AssetPlugin, Assets, Camera3dBundle, Color, Commands,
        DirectionalLight, DirectionalLightBundle, MaterialMeshBundle, Mesh, OrthographicProjection,
        PbrBundle, PluginGroup, Quat, ResMut, StandardMaterial, Transform, Vec3,
    },
    utils::HashSet,
    DefaultPlugins,
};
use capture_point::CapturePointPlugin;
use player::PlayerPlugin;
use ship::{space_ship::SpaceShip, SpaceShipPlugin};

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
