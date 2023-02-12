pub mod bullet;

use std::time::Instant;

use bevy::{
    prelude::{
        App, Commands, Component, GlobalTransform, IntoSystemDescriptor, Parent, Plugin, Quat,
        Query, Res, ResMut, Transform, With, Without,
    },
    time::Time,
    transform::TransformBundle,
};
use bevy_renet::renet::{DefaultChannel, RenetServer};
use spaaaace_shared::{player::player_input::PlayerInput, NetworkedId, ServerMessages};

use crate::Player;

use self::bullet::{Bullet, BulletPlugin};

#[derive(Component)]
pub struct Turret {
    pub fire_rate: f32,
    pub cooldown: f32,
    pub trigger: bool,
    pub aim_dir: Quat,
}
impl Default for Turret {
    fn default() -> Self {
        Self {
            fire_rate: 1.0,
            cooldown: Default::default(),
            trigger: Default::default(),
            aim_dir: Default::default(),
        }
    }
}

pub struct WeaponsPlugin;

impl Plugin for WeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(BulletPlugin {})
            .add_system(trigger_weapons)
            .add_system(turn_turrets)
            .add_system(fire_weapons.after(trigger_weapons));
    }
}

#[derive(Component)]
pub struct Barrel {}

fn trigger_weapons(
    mut q_child: Query<(&Parent, &mut Turret)>,
    q_parent: Query<(&Player, &PlayerInput)>,
) {
    for (parent, mut turret) in q_child.iter_mut() {
        let result = q_parent.get(parent.get());
        match result {
            Ok((_player, player_input)) => {
                turret.trigger = player_input.primary_fire;
            }
            Err(_) => {}
        }
    }
}

fn turn_turrets(
    time: Res<Time>,
    mut turret_query: Query<
        (&Parent, &mut Turret, &mut Transform, &GlobalTransform),
        Without<Barrel>,
    >,
    mut barrel_query: Query<(&Parent, &mut Transform, &GlobalTransform), With<Barrel>>,
    player_query: Query<(&Player, &PlayerInput)>,
) {
    for (parent, _, mut transform, global_transform) in turret_query.iter_mut() {
        let player = player_query.get(parent.get());
        match player {
            Ok((_player, player_input)) => {
                let direction =
                    (global_transform.translation() - player_input.aim_point).normalize_or_zero();
                let off_by = global_transform.right().dot(direction) * time.delta_seconds() * 5.;
                transform.rotate_local_y(off_by);
            }
            Err(_) => {}
        }
    }

    for (parent, mut transform, global_transform) in barrel_query.iter_mut() {
        let (turret_parent, _, _, _) = turret_query.get(parent.get()).unwrap();
        let player = player_query.get(turret_parent.get());

        match player {
            Ok((_player, player_input)) => {
                let direction =
                    (global_transform.translation() - player_input.aim_point).normalize_or_zero();
                let off_by = global_transform.down().dot(direction) * time.delta_seconds() * 5.0;
                transform.rotate_local_x(off_by);
            }
            Err(_) => {}
        }
    }
}

fn fire_weapons(
    mut barrel_query: Query<(&Barrel, &GlobalTransform, &Parent)>,
    mut turret_query: Query<&mut Turret>,
    mut commands: Commands,
    time: Res<Time>,
    mut server: ResMut<RenetServer>,
) {
    for (_, global_transform, parent) in barrel_query.iter_mut() {
        let mut turret = turret_query.get_mut(parent.get()).unwrap();
        if turret.cooldown <= 0.0 {
            if turret.trigger {
                let transform = global_transform.compute_transform();

                let now = Instant::now();
                let since_start = now.duration_since(time.startup());
                let id = since_start.as_nanos();

                commands
                    .spawn(TransformBundle::from_transform(transform))
                    .insert(Bullet {
                        speed: 200.,
                        lifetime: time.elapsed_seconds() + 2.0,
                    })
                    .insert(NetworkedId {
                        id: id.try_into().unwrap(),
                        last_sent: 0,
                    });

                let message = bincode::serialize(&ServerMessages::BulletSpawned {
                    id: id.try_into().unwrap(),
                    position: transform.translation,
                    rotation: transform.rotation,
                })
                .unwrap();

                server.broadcast_message(DefaultChannel::Reliable, message);
            }
            turret.cooldown = turret.fire_rate;
        } else {
            turret.cooldown -= time.delta_seconds();
        }
    }
}
