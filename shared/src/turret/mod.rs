pub mod bullet;
pub mod model_load_handlers;

use std::{f32::consts::PI, time::Instant};

use bevy::{
    prelude::{
        default, shape, App, Assets, Color, Commands, Component, Entity, EventReader,
        GlobalTransform, IntoSystemDescriptor, Mesh, PbrBundle, Plugin, Quat, Query, Res,
        ResMut, StandardMaterial, SystemSet, Transform, Vec3, With, Without,
    },
    time::Time,
    transform::TransformBundle,
};
use bevy_renet::renet::{DefaultChannel, RenetServer};

use crate::{
    cooldown::Cooldown,
    player::{player_input::PlayerInput},
    run_if_client, run_if_server,
    util::spring::SpringVelocity,
    Lobby, NetworkedId, ServerMessages,
};

use self::{
    bullet::{Bullet, BulletBundle, BulletPlugin},
    model_load_handlers::handle_turret_model_load,
};

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

#[derive(Component, Debug, Eq, PartialEq)]
pub struct PartOfTurret(pub(crate) Entity);

impl PartOfTurret {
    pub fn new(entity: Entity) -> PartOfTurret {
        PartOfTurret(entity)
    }

    pub fn get(&self) -> Entity {
        self.0
    }
}

#[derive(Component)]
pub struct Turret {
    pub fire_rate: f32,
    pub barrel_count: i8,
    pub cooldown: f32,
    pub trigger: bool,
    pub aim_dir: Quat,
}
impl Default for Turret {
    fn default() -> Self {
        Self {
            fire_rate: 3.0,
            barrel_count: 0,
            cooldown: Default::default(),
            trigger: Default::default(),
            aim_dir: Default::default(),
        }
    }
}

pub struct TurretPlugin;

impl Plugin for TurretPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(BulletPlugin {})
            .add_system(handle_turret_model_load)
            .add_system(trigger_weapons)
            .add_system(turn_turrets)
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(run_if_server)
                    .with_system(fire_weapons_server.after(trigger_weapons)),
            )
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(run_if_client)
                    .with_system(on_bullet_spawned_client),
            );
    }
}

#[derive(Component)]
pub struct TurretBase {}

#[derive(Component)]
pub struct TurretPivot {}

#[derive(Component)]
pub struct TurretBarrel {}

fn trigger_weapons(mut q_child: Query<(&mut Turret, &TurretOwner)>, q_parent: Query<&PlayerInput>) {
    for (mut turret, turret_owner) in q_child.iter_mut() {
        let result = q_parent.get(turret_owner.get());
        match result {
            Ok(player_input) => {
                turret.trigger = player_input.primary_fire;
            }
            Err(_) => {}
        }
    }
}

fn turn_turrets(
    time: Res<Time>,
    turret_query: Query<
        (&TurretOwner, &Turret, &Transform, &GlobalTransform),
        (Without<TurretBase>, Without<TurretPivot>),
    >,
    mut base_query: Query<
        (&PartOfTurret, &mut Transform, &GlobalTransform),
        (With<TurretBase>, Without<TurretPivot>),
    >,
    mut pivot_query: Query<
        (&PartOfTurret, &mut Transform, &GlobalTransform),
        (With<TurretPivot>, Without<TurretBase>),
    >,
    player_query: Query<&PlayerInput>,
) {
    for (part_of_turret, mut transform, global_transform) in base_query.iter_mut() {
        let (turret_parent, _, _, _) = turret_query.get(part_of_turret.get()).unwrap();
        let player = player_query.get(turret_parent.get());

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

    for (part_of_turret, mut transform, global_transform) in pivot_query.iter_mut() {
        let (turret_parent, _, _, _) = turret_query.get(part_of_turret.get()).unwrap();
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
    mut barrel_query: Query<
        (Entity, &TurretBarrel, &GlobalTransform, &PartOfTurret),
        Without<Cooldown>,
    >,
    mut turret_query: Query<(Entity, &Turret, Option<&Cooldown>)>,
    mut commands: Commands,
    time: Res<Time>,
    mut server: ResMut<RenetServer>,
) {
    let mut turrets_fired = Vec::<u32>::new();
    for (entity, _, global_transform, part_of_turret) in barrel_query.iter_mut() {
        let (turret_ent, turret, turret_cooldown) =
            turret_query.get_mut(part_of_turret.get()).unwrap();

        if turret_cooldown.is_none()
            && turret.trigger
            && turrets_fired.contains(&turret_ent.index()) == false
        {
            let transform = global_transform.compute_transform();

            let now = Instant::now();
            let since_start = now.duration_since(time.startup());
            let id = since_start.as_nanos();

            let bullet_transform = TransformBundle::from_transform(transform);
            let bullet = Bullet {
                speed: 20.,
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
            commands.entity(entity).insert((
                Cooldown { value: 0.6 },
                SpringVelocity {
                    value: Vec3::Z * 10.0,
                },
            ));
            commands.entity(turret_ent).insert(Cooldown { value: 0.3 });
            turrets_fired.push(turret_ent.index());
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
                            base_color: Color::BLACK,
                            perceptual_roughness: 1.,
                            emissive: Color::rgb(1., 0.2, 0.2) * 5.,
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
