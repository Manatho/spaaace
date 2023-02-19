use std::f32::consts::PI;

use bevy::{
    prelude::{
        shape, App, AssetServer, Assets, Color, Commands, EventReader, Mesh, PbrBundle, Plugin,
        Quat, Res, ResMut, StandardMaterial, Transform,
    },
    scene::SceneBundle,
    utils::default,
};

use bevy_rapier3d::prelude::Collider;
use spaaaace_shared::{Lobby, ServerMessages};

pub struct AsteroidPlugin;
impl Plugin for AsteroidPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(on_asteroid_spawned);
    }
}

fn on_asteroid_spawned(
    mut commands: Commands,
    mut lobby: ResMut<Lobby>,
    mut event_reader: EventReader<ServerMessages>,
    ass: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for event in event_reader.iter() {
        match event {
            ServerMessages::AsteroidSpawned {
                id,
                position,
                scale,
                rotation,
            } => {
                let x = commands
                    .spawn(SceneBundle {
                        scene: ass.load("asteroid.glb#Scene0"),
                        transform: Transform {
                            translation: *position,
                            scale: *scale,
                            rotation: *rotation,
                            ..default()
                        },
                        ..default()
                    })
                    // .spawn(PbrBundle {
                    //     mesh: meshes.add(Mesh::from(shape::Capsule {
                    //         radius: 5.0,
                    //         ..Default::default()
                    //     })),
                    //     material: materials.add(StandardMaterial {
                    //         base_color: Color::BLACK,
                    //         ..default()
                    //     }),
                    //     transform: Transform {
                    //         translation: *position,
                    //         rotation: *rotation * Quat::from_rotation_x(PI / 2.0),
                    //         ..Default::default()
                    //     },
                    //     ..Default::default()
                    // })
                    .insert(Collider::ball(5.0))
                    .id();

                println!("{} {}", *id, x.index());

                lobby.networked_entities.insert(*id, x);
            }
            _ => {}
        }
    }
}
