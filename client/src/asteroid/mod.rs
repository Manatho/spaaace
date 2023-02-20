use bevy::{
    prelude::{App, AssetServer, Commands, EventReader, Plugin, Res, ResMut, Transform},
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
                    .insert(Collider::ball(1.0))
                    .id();

                println!("{} {}", *id, x.index());

                lobby.networked_entities.insert(*id, x);
            }
            _ => {}
        }
    }
}
