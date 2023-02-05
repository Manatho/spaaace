use bevy::{
    prelude::{App, Commands, Component, Entity, Plugin, Query, Res, ResMut, Transform},
    time::Time,
};
use bevy_renet::renet::{DefaultChannel, RenetServer};
use spaaaace_shared::{NetworkedId, ServerMessages};

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(bullet_mover).add_system(bullet_remover);
    }
}

#[derive(Component)]
pub struct Bullet {
    pub speed: f32,
    pub lifetime: f32,
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
