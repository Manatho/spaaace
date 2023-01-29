use bevy::{
    ecs::system::Command,
    math::{vec2, vec3},
    prelude::{
        default, shape, App, Assets, BuildChildren, Bundle, ChildBuilder, Color, Commands,
        Component, EulerRot, GlobalTransform, IntoSystemDescriptor, Mesh, Parent, PbrBundle,
        Plugin, Quat, Query, Res, ResMut, StandardMaterial, Transform, Vec2, Vec3, With, Without,
    },
    time::Time,
    transform::TransformBundle,
};
use bevy_mod_gizmos::{draw_gizmo, Gizmo};
use bevy_renet::renet::{DefaultChannel, RenetServer};
use spaaaace_shared::{player::player_input::PlayerInput, ServerMessages};

use crate::Player;

#[derive(Component)]
pub struct Turret {
    pub fire_rate: f32,
    pub cooldown: f32,
    pub trigger: bool,
    pub aim_dir: Quat,
}

#[derive(Component)]
pub struct Projectile;

pub struct WeaponsPlugin;

impl Plugin for WeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(move_projectiles)
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
                    (transform.translation - player_input.aim_point).normalize_or_zero();
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
                    (transform.translation - player_input.aim_point).normalize_or_zero();
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

                commands.spawn(TransformBundle::from_transform(transform));

                let message = bincode::serialize(&ServerMessages::BulletSpawned {
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

fn move_projectiles(mut query: Query<&mut Transform, With<Projectile>>, time: Res<Time>) {
    for mut transform in query.iter_mut() {
        let forward = transform.forward();
        transform.translation += forward * time.delta_seconds();
    }
}
