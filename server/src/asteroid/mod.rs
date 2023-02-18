use bevy::{
    prelude::{
        App, Commands, Component, EventReader, Plugin, Quat,
        Query, ResMut, Transform, Vec3, With,
    },
    transform::TransformBundle,
};
use bevy_rapier3d::prelude::{
    Collider, ColliderMassProperties, Damping, GravityScale, RigidBody, Sleeping,
};
use bevy_renet::renet::{DefaultChannel, RenetServer, ServerEvent};
use rand::Rng;
use spaaaace_shared::{util::Random, NetworkIdProvider, NetworkedId, ServerMessages};

#[derive(Component)]
pub struct Asteroid;

pub struct AsteroidPlugin;

impl Plugin for AsteroidPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_asteroids)
            .add_system(on_client_connected);
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

        println!(
            "{} {} {}",
            x.translation.x, x.translation.y, x.translation.z
        );

        // commands
        // .spawn(id_provider.new_id())
        // .insert(Collider::cuboid(1.0, 1.0, 1.0))
        // .insert(RigidBody::Dynamic)
        // .insert(Asteroid)
        // .insert(TransformBundle {
        //     global: x.into(),
        //     ..Default::default()
        // });

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
            .insert(Asteroid);
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
