use bevy::prelude::{App, Commands, Component, Entity, EventReader, Plugin, Query, ResMut};
use bevy_rapier3d::prelude::CollisionEvent;
use bevy_renet::renet::{DefaultChannel, RenetServer};

use crate::{weapons::bullet::Bullet, NetworkedId, ServerMessages};

#[derive(Component)]
pub struct Health {
    pub health: f32,
}

pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(handle_collisions);
        app.add_system(death);
    }
}

fn handle_collisions(
    mut collision_event_reader: EventReader<CollisionEvent>,
    mut health_query: Query<(Entity, &mut Health)>,
    bullet_query: Query<&Bullet>,
) {
    for event in collision_event_reader.iter() {
        match event {
            CollisionEvent::Started(e1, e2, _) => {
                let result = health_query.iter_many([*e1, *e2]);
                let mut health_entity: Option<Entity> = Option::None;
                for (entity, _) in result {
                    health_entity = Some(entity);
                }

                let result = bullet_query.iter_many([*e1, *e2]);
                let mut bullet: Option<&Bullet> = Option::None;
                for b in result.into_iter() {
                    bullet = Some(b);
                }

                if health_entity.is_some() && bullet.is_some() {
                    let (_, mut health) = health_query.get_mut(health_entity.unwrap()).unwrap();
                    health.health -= 1.0;
                }
            }
            _ => (),
        }
    }
}

fn death(
    mut commands: Commands, //
    health_query: Query<(Entity, &mut Health, &NetworkedId)>,
    mut server: ResMut<RenetServer>,
) {
    for (entity, health, networked_id) in health_query.iter() {
        if health.health < 0.0 {
            commands.entity(entity).despawn();
            let message = bincode::serialize(&ServerMessages::EntityDespawn {
                id: networked_id.id,
            })
            .unwrap();
            server.broadcast_message(DefaultChannel::Reliable, message);
        }

        print!("{} : ", health.health);
    }
    println!()
}
