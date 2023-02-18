use std::f32::consts::PI;

use bevy::{
    prelude::{
        shape, App, Assets, Color, Commands, Component, EventReader, Mesh, PbrBundle, Plugin, Quat,
        ResMut, StandardMaterial, Transform,
    },
    utils::default,
};

use spaaaace_shared::{Lobby, ServerMessages};

pub struct ClientWeaponsPlugin;

impl Plugin for ClientWeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(on_bullet_spawned);
    }
}

#[derive(Component)]
struct Bullet {}

fn on_bullet_spawned(
    mut commands: Commands,
    mut lobby: ResMut<Lobby>,
    mut event_reader: EventReader<ServerMessages>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for event in event_reader.iter() {
        match event {
            ServerMessages::BulletSpawned {
                id,
                position,
                rotation,
            } => {
                let entity_id = commands
                    .spawn(PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Capsule {
                            depth: 0.5,
                            radius: 0.1,
                            ..Default::default()
                        })),
                        material: materials.add(StandardMaterial {
                            base_color: Color::BLACK,
                            perceptual_roughness: 1.,
                            emissive: Color::rgb(1., 0.2, 0.2) * 5.,
                            ..default()
                        }),
                        transform: Transform {
                            translation: *position,
                            rotation: *rotation * Quat::from_rotation_x(PI / 2.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .insert(Bullet {})
                    .id();

                lobby.networked_entities.insert(*id, entity_id);
            }
            _ => {}
        }
    }
}
