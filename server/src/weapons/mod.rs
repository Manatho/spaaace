use bevy::{
    prelude::{
        App, Children, Commands, Component, GlobalTransform, IntoSystemDescriptor, Parent, Plugin,
        Quat, Query, Res, ResMut, Transform, With,
    },
    time::Time,
    transform::TransformBundle,
};
use bevy_renet::renet::{DefaultChannel, RenetServer};
use spaaaace_shared::{PlayerInput, ServerMessages};

use crate::Player;

#[derive(Component)]
pub struct Turret {
    pub fire_rate: f32,
    pub cooldown: f32,
    pub trigger: bool,
}

#[derive(Component)]
pub struct Projectile;

pub struct WeaponsPlugin;

impl Plugin for WeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(move_projectiles)
            .add_system(trigger_weapons)
            .add_system(fire_weapons.after(trigger_weapons));
    }
}

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

fn fire_weapons(
    mut query: Query<(&mut Turret, &GlobalTransform), With<Parent>>,
    mut commands: Commands,
    time: Res<Time>,
    mut server: ResMut<RenetServer>,
) {
    for (mut turret, transform) in query.iter_mut() {
        if turret.cooldown <= 0.0 {
            if turret.trigger {
                let translation = transform.translation();
                let rotation = Quat::IDENTITY;
                commands.spawn(TransformBundle::from_transform(Transform {
                    translation,
                    rotation,
                    ..Default::default()
                }));
                println!("{}", translation);
                let message = bincode::serialize(&ServerMessages::BulletSpawned {
                    position: translation,
                    rotation: rotation,
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
