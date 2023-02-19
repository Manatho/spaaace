use bevy::{
    prelude::{App, Bundle, Commands, Component, Entity, Plugin, Query, Res, ResMut, Transform},
    time::Time,
};
use bevy_rapier3d::prelude::{ActiveEvents, Collider, CollisionGroups, Group, Sensor};
use bevy_renet::renet::{DefaultChannel, RenetServer};
use spaaaace_shared::{NetworkedId, ServerMessages, asteroid::Bullet};

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(bullet_mover).add_system(bullet_remover);
    }
}

#[derive(Bundle)]
pub struct BulletBundle {
    pub bullet: Bullet,
    active_events: ActiveEvents,
    collider: Collider,
    sensor: Sensor,
}

impl BulletBundle {
    pub fn new(bullet: Bullet) -> Self {
        BulletBundle {
            bullet,
            active_events: ActiveEvents::COLLISION_EVENTS,
            collider: Collider::ball(0.5),
            sensor: Sensor,
        }
    }
}

fn bullet_mover(
    mut query: Query<(&mut Transform, &Bullet)>, //
    time: Res<Time>,
) {
    for (mut transform, bullet) in query.iter_mut() {
        let dir = transform.forward();
        transform.translation += dir * time.delta_seconds() * bullet.speed;
    }
}

fn bullet_remover(
    mut commands: Commands,
    mut query: Query<(Entity, &Bullet, &NetworkedId)>, //
    time: Res<Time>,
    mut server: ResMut<RenetServer>,
) {
    for (entity, bullet, networked_id) in query.iter_mut() {
        if time.elapsed_seconds() > bullet.lifetime {
            commands.entity(entity).despawn();

            let message = bincode::serialize(&ServerMessages::EntityDespawn {
                id: networked_id.id,
            })
            .unwrap();
            server.broadcast_message(DefaultChannel::Reliable, message);
        }
    }
}
