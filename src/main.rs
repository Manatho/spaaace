use std::f32::consts::PI;

use bevy::{
    prelude::{
        default, shape, App, Assets, Camera3dBundle, Color, Commands, Component, DirectionalLight,
        DirectionalLightBundle, Mesh, OrthographicProjection, PbrBundle, Quat, Query, ResMut,
        StandardMaterial, Transform, Vec3, Res,
    },
    DefaultPlugins, time::Time,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(forever_move)
        .run();
}

fn forever_move(mut query: Query<(&mut Transform, &SpaceShip)>, time: Res<Time>) {
    for (mut transform, spaceship) in query.iter_mut() {
        let forward = transform.forward().clone();
        transform.translation += forward * time.delta_seconds();
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane { size: 50. }.into()),
        material: materials.add(Color::SILVER.into()),
        ..default()
    });

    commands
        .spawn(PbrBundle {
            mesh: meshes.add(shape::Cube { size: 1. }.into()),
            material: materials.add(Color::ALICE_BLUE.into()),
            ..default()
        })
        .insert(SpaceShip { hp: 20 });

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

#[derive(Component)]
struct SpaceShip {
    hp: i32,
}
