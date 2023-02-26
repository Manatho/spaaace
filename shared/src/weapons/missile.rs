use bevy::{
    prelude::{
        App, Bundle, Commands, Component, Entity, Plugin, Query, Res, ResMut, SystemSet, Transform,
    },
    time::Time,
};
use bevy_rapier3d::prelude::{ActiveEvents, Collider, Sensor};
use bevy_renet::renet::{DefaultChannel, RenetServer};

use crate::{run_if_server, NetworkedId, ServerMessages, player::{Player, player_input::PlayerInput}};

pub struct MissilePlugin;

impl Plugin for MissilePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(missile_mover).add_system_set(
            SystemSet::new()
                .with_run_criteria(run_if_server)
                .with_system(missile_remover),
        );
    }
}
#[derive(Component, Clone, Copy)]
pub struct Missile {
    pub speed: f32,
    pub lifetime: f32,
}

#[derive(Bundle)]
pub struct MissileBundle {
    pub missile: Missile,
    active_events: ActiveEvents,
    collider: Collider,
    sensor: Sensor,
}

impl MissileBundle {
    pub fn new(missile: Missile) -> Self {
        MissileBundle {
            missile,
            active_events: ActiveEvents::COLLISION_EVENTS,
            collider: Collider::ball(0.5),
            sensor: Sensor,
        }
    }
}

fn fire_missile(
    input_query: Query<(&Player, &PlayerInput)>,
    time: Res<Time>,
) {
    for (player, mut input_query) in input_query.iter_mut() {
        
        match result {
            Ok((_player, player_input)) => {
                turret.trigger = player_input.primary_fire;
            }
            Err(_) => {}
        }
    }
}

fn missile_mover(
    mut query: Query<(&mut Transform, &Missile)>, //
    time: Res<Time>,
) {
    for (mut transform, Missile) in query.iter_mut() {
        let dir = transform.forward();
        transform.translation += dir * time.delta_seconds() * Missile.speed;
    }
}

fn missile_remover(
    mut commands: Commands,
    mut query: Query<(Entity, &Missile, &NetworkedId)>,
    mut server: ResMut<RenetServer>,
    time: Res<Time>,
) {
    for (entity, Missile, networked_id) in query.iter_mut() {
        if time.elapsed_seconds() > Missile.lifetime {
            commands.entity(entity).despawn();

            let message = bincode::serialize(&ServerMessages::EntityDespawn {
                id: networked_id.id,
            })
            .unwrap();
            server.broadcast_message(DefaultChannel::Reliable, message);
        }
    }
}
