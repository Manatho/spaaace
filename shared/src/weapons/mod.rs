pub mod bullet;

use std::{f32::consts::PI, time::Instant};

use bevy::{
    prelude::{
        default, shape, App, Assets, Color, Commands, Component, Entity, EventReader,
        GlobalTransform, IntoSystemConfig, Mesh, Parent, PbrBundle, Plugin, Quat, Query, Res,
        ResMut, StandardMaterial, SystemSet, Transform, With, Without,
    },
    time::Time,
    transform::TransformBundle,
};
use bevy_renet::renet::{DefaultChannel, RenetServer};

use crate::{
    player::{player_input::PlayerInput, Player},
    run_if_client, run_if_server, Lobby, NetworkedId, ServerMessages,
};

use self::bullet::{Bullet, BulletBundle, BulletPlugin};

#[derive(Component, Debug, Eq, PartialEq)]
pub struct TurretOwner(pub(crate) Entity);

impl TurretOwner {
    pub fn new(entity: Entity) -> TurretOwner {
        TurretOwner(entity)
    }

    pub fn get(&self) -> Entity {
        self.0
    }
}

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
            .add_systems((trigger_weapons, turn_turrets))
            .add_system(
                fire_weapons_server
                    .after(trigger_weapons)
                    .run_if(run_if_server),
            )
            .add_system(on_bullet_spawned_client.run_if(run_if_client));
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
        (&TurretOwner, &mut Turret, &mut Transform, &GlobalTransform),
        Without<Barrel>,
    >,
    mut barrel_query: Query<(&Parent, &mut Transform, &GlobalTransform), With<Barrel>>,
    player_query: Query<&PlayerInput>,
) {
    for (owner, _, mut transform, global_transform) in turret_query.iter_mut() {
        let player = player_query.get(owner.get());

        match player {
            Ok(player_input) => {
                let direction =
                    (global_transform.translation() - player_input.aim_point).normalize_or_zero();
                let off_by = global_transform.right().dot(direction) * time.delta_seconds() * 10.0;
                transform.rotate_local_y(off_by);
            }
            Err(x) => println!("Turret has not player parent: {}", x),
        }
    }

    for (parent, mut transform, global_transform) in barrel_query.iter_mut() {
        let (turret_parent, _, _, _) = turret_query.get(parent.get()).unwrap();
        let player = player_query.get(turret_parent.get());

        match player {
            Ok(player_input) => {
                let direction =
                    (global_transform.translation() - player_input.aim_point).normalize_or_zero();
                let off_by = global_transform.down().dot(direction) * time.delta_seconds() * 10.0;
                transform.rotate_local_x(off_by);
            }
            Err(_) => {}
        }
    }
}

fn fire_weapons_server(
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

                let bullet_transform = TransformBundle::from_transform(transform);
                let bullet = Bullet {
                    speed: 200.,
                    lifetime: time.elapsed_seconds() + 2.0,
                };
                commands
                    .spawn(bullet_transform)
                    .insert(BulletBundle::new(bullet))
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

fn on_bullet_spawned_client(
    mut commands: Commands,
    mut lobby: ResMut<Lobby>,
    mut event_reader: EventReader<ServerMessages>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for event in event_reader.iter() {
        match event {
            ServerMessages::BulletSpawned {
                id,
                position,
                rotation,
            } => {
                let entity_id = commands
                    .spawn(PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Capsule {
                            depth: 0.5,
                            radius: 0.1,
                            ..Default::default()
                        })),
                        material: materials.add(StandardMaterial {
                            emissive: Color::rgb_linear(13.99, 5.32, 2.0) * 5.,
                            ..default()
                        }),
                        transform: Transform {
                            translation: *position,
                            rotation: *rotation * Quat::from_rotation_x(PI / 2.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .insert(Bullet {
                        lifetime: 0.0,
                        speed: 0.0,
                    })
                    .id();

                lobby.networked_entities.insert(*id, entity_id);
            }
            _ => {}
        }
    }
}
