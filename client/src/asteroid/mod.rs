use bevy::{
    prelude::{
        App, AssetServer, Commands, EventReader, Plugin, Res, ResMut, Transform,
    },
    scene::SceneBundle,
    utils::default,
};

use spaaaace_shared::{Lobby, ServerMessages};

pub struct ClientCapturePointPlugin;
impl Plugin for ClientCapturePointPlugin {
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
                    .id();

                lobby.networked_entities.insert(*id, x);
            }
            _ => {}
        }
    }
}
