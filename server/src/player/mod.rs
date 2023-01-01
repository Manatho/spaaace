use bevy::{
    math::vec3,
    prelude::{
        App, BuildChildren, Color, Commands, Component, CoreStage, DespawnRecursiveExt,
        EventReader, Plugin, Quat, Query, ResMut, SystemStage, Transform, Vec3,
    },
    time::FixedTimestep,
    transform::TransformBundle,
    utils::HashMap,
};
use bevy_mod_gizmos::{draw_gizmo, Gizmo};
use bevy_rapier3d::prelude::{Collider, Damping, ExternalForce, GravityScale, RigidBody};

use bevy_renet::renet::{DefaultChannel, RenetServer, ServerEvent};
use spaaaace_shared::{
    team::team_enum::Team, ClientMessages, Lobby, PlayerInput, ServerMessages, TranslationRotation,
    SERVER_TICKRATE,
};

use crate::{weapons::Turret, ClientEvent, FixedUpdateStage};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update_players_system)
            .add_system(swap_team_command)
            .add_system(player_input)
            .add_system(on_client_disconnected)
            .add_system(on_client_connected)
            .add_system(draw_player_gizmos)
            .add_stage_after(
                CoreStage::Update,
                FixedUpdateStage,
                SystemStage::parallel()
                    .with_run_criteria(FixedTimestep::step(1.0 / (SERVER_TICKRATE as f64)))
                    .with_system(server_sync_players),
            );
    }
}

const PLAYER_MOVE_SPEED: f32 = 2.0;

#[derive(Component, Clone, Hash, PartialEq, Eq)]
pub struct Player {
    pub id: u64,
    pub team: Team,
}

fn update_players_system(mut query: Query<(&mut ExternalForce, &Transform, &PlayerInput)>) {
    for (mut rigidbody, transform, input) in query.iter_mut() {
        let rotation = (input.rotate_right as i8 - input.rotate_left as i8) as f32;
        let thrust_longitudal = (input.thrust_forward as i8 - input.thrust_reverse as i8) as f32;
        let thrust_lateral = (input.thrust_left as i8 - input.thrust_right as i8) as f32;
        let thrust_vertical = (input.thrust_up as i8 - input.thrust_down as i8) as f32;

        let forward = transform.forward();
        let projected_forward = (forward - Vec3::new(0.0, forward.y, 0.0)).normalize();
        let rotated_forward =
            (Quat::from_axis_angle(transform.left(), -0.6 * thrust_vertical)) * projected_forward;

        let left = transform.left();
        let projected_left = (left - Vec3::new(0.0, left.y, 0.0)).normalize();

        let longitudal_force = thrust_longitudal * PLAYER_MOVE_SPEED * 20.0 * projected_forward;
        let lateral_force = thrust_lateral * PLAYER_MOVE_SPEED * 5.0 * projected_left;
        let vertical_force = thrust_vertical * PLAYER_MOVE_SPEED * 10.0 * Vec3::Y;

        draw_gizmo(Gizmo::cubiod(
            transform.translation + rotated_forward * 2.0,
            vec3(0.3, 0.3, 0.3),
            Color::PURPLE,
        ));

        draw_gizmo(Gizmo::cubiod(
            transform.translation + transform.forward() * 2.5,
            vec3(0.3, 0.3, 0.3),
            Color::GREEN,
        ));

        rigidbody.force = longitudal_force + lateral_force + vertical_force;
        rigidbody.torque = rotation * Vec3::NEG_Y * PLAYER_MOVE_SPEED * 2.0;

        {
            let (axis, angle) =
                Quat::from_rotation_arc(transform.forward(), rotated_forward).to_axis_angle();
            rigidbody.torque += axis.normalize_or_zero() * angle;
        }

        {
            let (axis, angle) = Quat::from_rotation_arc(transform.up(), Vec3::Y).to_axis_angle();
            rigidbody.torque += axis.normalize_or_zero() * angle * 10.0;
        }
    }
}

fn server_sync_players(mut server: ResMut<RenetServer>, query: Query<(&Transform, &Player)>) {
    let mut players: HashMap<u64, TranslationRotation> = HashMap::new();
    for (transform, player) in query.iter() {
        players.insert(
            player.id,
            TranslationRotation {
                translation: transform.translation,
                rotation: transform.rotation,
            },
        );
    }

    let sync_message = bincode::serialize(&players).unwrap();
    server.broadcast_message(DefaultChannel::Unreliable, sync_message);
}

fn swap_team_command(
    mut client_message_event_reader: EventReader<ClientEvent>,
    lobby: ResMut<Lobby>,
    mut player_query: Query<&mut Player>,
) {
    for event in client_message_event_reader.iter() {
        match event.message.clone() {
            ClientMessages::Command { command } => {
                let args_split = command.split(" ");
                let args: Vec<&str> = args_split.collect();

                match args[0] {
                    "swap_team" => {
                        let entity = lobby.players[&event.client_id];

                        match player_query.get_mut(entity) {
                            Ok(mut player) => match args[1] {
                                "1" => player.team = Team::Red,
                                "2" => player.team = Team::Blue,
                                _ => (),
                            },
                            Err(_) => (),
                        }
                    }
                    _ => (),
                }
            }
            _ => (),
        }
    }
}

fn player_input(
    mut client_message_event_reader: EventReader<ClientEvent>,
    mut commands: Commands,
    lobby: ResMut<Lobby>,
) {
    for event in client_message_event_reader.iter() {
        match event.message.clone() {
            ClientMessages::PlayerInput { input } => {
                if let Some(player_entity) = lobby.players.get(&event.client_id) {
                    commands.entity(*player_entity).insert(input);
                }
            }
            _ => (),
        }
    }
}

fn on_client_disconnected(
    mut event_reader: EventReader<ServerEvent>,
    mut commands: Commands,
    mut lobby: ResMut<Lobby>,
    mut server: ResMut<RenetServer>,
) {
    for event in event_reader.iter() {
        match event {
            ServerEvent::ClientDisconnected(id) => {
                println!("Player {} disconnected.", id);
                if let Some(player_entity) = lobby.players.remove(id) {
                    commands.entity(player_entity).despawn_recursive();
                }

                let message =
                    bincode::serialize(&ServerMessages::PlayerDisconnected { id: *id }).unwrap();
                server.broadcast_message(DefaultChannel::Reliable, message);
            }
            _ => (),
        }
    }
}

fn on_client_connected(
    mut event_reader: EventReader<ServerEvent>,
    mut commands: Commands,
    mut lobby: ResMut<Lobby>,
    mut server: ResMut<RenetServer>,
) {
    for event in event_reader.iter() {
        match event {
            ServerEvent::ClientConnected(id, _) => {
                println!("Player {} connected.", id);
                // Spawn player cube
                let player_entity = commands
                    .spawn(TransformBundle {
                        local: Transform {
                            translation: vec3(0.0, 0.5, 0.0),
                            rotation: Quat::from_rotation_x(0.5),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .insert(PlayerInput::default())
                    .insert(Player {
                        id: *id,
                        team: Team::Red,
                    })
                    .insert(Collider::cuboid(1.0, 1.0, 1.0))
                    .insert(RigidBody::Dynamic)
                    // .insert(LockedAxes::ROTATION_LOCKED_Z)
                    .insert(GravityScale(0.0))
                    .insert(ExternalForce::default())
                    .insert(Damping {
                        linear_damping: 0.5,
                        angular_damping: 1.0,
                    })
                    .with_children(|parent| {
                        println!("spawning turret");
                        parent
                            .spawn(TransformBundle {
                                ..Default::default()
                            })
                            .insert(Turret {
                                cooldown: 0.0,
                                fire_rate: 1.0 / 5.0,
                                trigger: false,
                            });
                    })
                    .id();

                // We could send an InitState with all the players id and positions for the client
                // but this is easier to do.
                for &player_id in lobby.players.keys() {
                    let message =
                        bincode::serialize(&ServerMessages::PlayerConnected { id: player_id })
                            .unwrap();
                    server.send_message(*id, DefaultChannel::Reliable, message);
                }

                lobby.players.insert(*id, player_entity);

                let message =
                    bincode::serialize(&ServerMessages::PlayerConnected { id: *id }).unwrap();
                server.broadcast_message(DefaultChannel::Reliable, message);
            }
            _ => (),
        }
    }
}

fn draw_player_gizmos(query: Query<(&Player, &Transform)>) {
    for (_, transform) in query.iter() {
        draw_gizmo(Gizmo::sphere(transform.translation, 1.0, Color::RED))
    }
}
