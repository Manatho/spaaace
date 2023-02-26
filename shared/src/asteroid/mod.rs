use bevy::{
    prelude::{
        default, App, AssetServer, Commands, Component, EventReader, Plugin, Quat, Query, Res,
        ResMut, SystemSet, Transform, Vec3, With,
    },
    scene::SceneBundle,
    transform::TransformBundle,
};
use bevy_rapier3d::prelude::{
    Collider, ColliderMassProperties, Damping, GravityScale, RigidBody, Sleeping,
};
use bevy_renet::renet::{DefaultChannel, RenetServer, ServerEvent};
use rand::Rng;

use crate::{
    health::Health, run_if_client, run_if_server, targeting::Targetable, util::Random, Lobby,
    NetworkIdProvider, NetworkedId, ServerMessages,
};

#[derive(Component)]
pub struct Asteroid;

pub struct AsteroidPlugin;

impl Plugin for AsteroidPlugin {
    fn build(&self, app: &mut App) {
        //Both

        //Client
        app.add_system_set(
            SystemSet::new()
                .with_run_criteria(run_if_client)
                .with_system(on_asteroid_spawned),
        );

        // Server
        app.add_system_set(
            SystemSet::new()
                .with_run_criteria(run_if_server)
                .with_system(on_client_connected),
        )
        .add_startup_system_set(
            SystemSet::new()
                .with_run_criteria(run_if_server)
                .with_system(spawn_asteroids),
        );
    }
}

fn spawn_asteroids(
    mut commands: Commands, //
    mut id_provider: ResMut<NetworkIdProvider>,
) {
    let mut rng = rand::thread_rng();
    for _ in 0..20 {
        let x = Transform {
            translation: Vec3 {
                x: rng.gen::<f32>() * 250.0,
                y: rng.gen::<f32>() * 250.0,
                z: rng.gen::<f32>() * 250.0,
            },
            scale: Vec3::splat(2.0 + rng.gen::<f32>() * 8.0),
            rotation: Quat::random(),
        };

        commands
            .spawn(Collider::ball(1.0))
            .insert(RigidBody::Dynamic)
            .insert(GravityScale(0.0))
            .insert(TransformBundle::from(x))
            .insert(Damping {
                angular_damping: 1.,
                linear_damping: 1.,
            })
            .insert(Sleeping {
                angular_threshold: 100.0,
                linear_threshold: 100.0,
                sleeping: true,
            })
            .insert(ColliderMassProperties::Density(1.0))
            .insert(id_provider.new_id())
            .insert(Asteroid)
            .insert(Health { health: 10.0 });
    }
}

fn on_client_connected(
    mut event_reader: EventReader<ServerEvent>,
    mut server: ResMut<RenetServer>,
    query: Query<(&Transform, &NetworkedId), With<Asteroid>>,
) {
    for event in event_reader.iter() {
        match event {
            ServerEvent::ClientConnected(id, _) => {
                for (transform, network_id) in query.iter() {
                    let message = bincode::serialize(&ServerMessages::AsteroidSpawned {
                        id: network_id.id,
                        position: transform.translation,
                        scale: transform.scale,
                        rotation: transform.rotation,
                    })
                    .unwrap();
                    server.send_message(*id, DefaultChannel::Reliable, message);
                }
            }
            _ => (),
        }
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
                    .insert(Targetable {})
                    .insert(NetworkedId {
                        id: *id,
                        last_sent: 0,
                    })
                    .insert(Collider::ball(1.0))
                    .id();

                lobby.networked_entities.insert(*id, x);
            }
            _ => {}
        }
    }
}
