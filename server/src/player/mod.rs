use std::path::Path;

use bevy::{
    gltf::{Gltf, GltfNode},
    math::vec3,
    prelude::{
        default, App, AssetServer, Assets, BuildChildren, Color, Commands, DespawnRecursiveExt,
        Entity, EventReader, PbrBundle, Plugin, Quat, Query, Res, ResMut, SpatialBundle, Transform,
        Vec3,
    },
    scene::SceneBundle,
    time::Time,
    transform::TransformBundle,
    utils::{HashMap, Instant},
};
use bevy_mod_gizmos::{draw_gizmo, Gizmo};
use bevy_rapier3d::prelude::{
    Collider, ColliderMassProperties, CollisionGroups, Damping, ExternalImpulse, GravityScale,
    Group, RigidBody, Sleeping,
};

use bevy_renet::renet::{DefaultChannel, RenetServer, ServerEvent};
use spaaaace_shared::{
    player::{player_input::PlayerInput, Player},
    ships::{ShipModelLoadHandle, SHIP_TYPES},
    team::team_enum::Team,
    weapons::{Barrel, Turret, TurretOwner},
    ClientMessages, Lobby, NetworkedId, ServerMessages, TranslationRotation,
};

use crate::ClientEvent;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update_players_system)
            .add_system(swap_team_command)
            .add_system(player_input)
            .add_system(on_client_disconnected)
            .add_system(on_client_connected)
            .add_system(on_client_model_loaded)
            .add_system(server_sync_players);
    }
}

const PLAYER_MOVE_SPEED: f32 = 2.0;

fn update_players_system(mut query: Query<(&mut ExternalImpulse, &Transform, &PlayerInput)>) {
    for (mut rigidbody, transform, input) in query.iter_mut() {
        let rotation = (input.rotate_right as i8 - input.rotate_left as i8) as f32;
        let thrust_longitudal = (input.thrust_forward as i8 - input.thrust_reverse as i8) as f32;
        let thrust_lateral = (input.thrust_left as i8 - input.thrust_right as i8) as f32;
        let thrust_vertical = (input.thrust_up as i8 - input.thrust_down as i8) as f32;

        let forward = transform.forward();
        let projected_forward = (forward - Vec3::new(0.0, forward.y, 0.0)).normalize();
        let rotated_forward =
            (Quat::from_axis_angle(transform.left(), -0.3 * thrust_vertical)) * projected_forward;

        let left = transform.left();
        let projected_left = (left - Vec3::new(0.0, left.y, 0.0)).normalize();

        let longitudal_force = thrust_longitudal * PLAYER_MOVE_SPEED * 50.0 * projected_forward;
        let lateral_force = thrust_lateral * PLAYER_MOVE_SPEED * 30.0 * projected_left;
        let vertical_force = thrust_vertical * PLAYER_MOVE_SPEED * 30.0 * Vec3::Y;

        draw_gizmo(Gizmo::new(
            transform.translation + rotated_forward * 2.0,
            0.3,
            Color::PURPLE,
        ));

        draw_gizmo(Gizmo::new(
            transform.translation + transform.forward() * 2.5,
            0.3,
            Color::GREEN,
        ));

        rigidbody.impulse = longitudal_force + lateral_force + vertical_force;
        rigidbody.torque_impulse = rotation * Vec3::NEG_Y * PLAYER_MOVE_SPEED * 60.0;

        {
            let (axis, angle) =
                Quat::from_rotation_arc(transform.forward(), rotated_forward).to_axis_angle();
            rigidbody.torque_impulse += axis.normalize_or_zero() * angle * 200.0;
        }

        {
            let (axis, angle) = Quat::from_rotation_arc(transform.up(), Vec3::Y).to_axis_angle();
            rigidbody.torque_impulse += axis.normalize_or_zero() * angle * 300.0;
        }
    }
}

fn server_sync_players(
    mut server: ResMut<RenetServer>,
    mut query: Query<(&Transform, &mut NetworkedId, Option<&Sleeping>)>,
    time: Res<Time>,
) {
    let mut entries: Vec<(&NetworkedId, TranslationRotation)> = Vec::new();

    for (transform, network_id, sleeping) in query.iter() {
        if sleeping.is_some() && sleeping.unwrap().sleeping {
            continue;
        }
        entries.push((
            network_id,
            TranslationRotation {
                translation: transform.translation,
                rotation: transform.rotation,
            },
        ));
    }

    entries.sort_by(|(a, _), (b, _)| a.last_sent.cmp(&b.last_sent));

    let mut messages: HashMap<u64, TranslationRotation> = HashMap::new();

    for (id, tr) in entries {
        if messages.len() < 70 {
            messages.insert(id.id, tr);
        } else {
            break;
        }
    }

    let sync_message = bincode::serialize(&messages).unwrap();
    server.broadcast_message(DefaultChannel::Unreliable, sync_message);

    for (_, mut network_id, sleeping) in query.iter_mut() {
        if sleeping.is_some() && sleeping.unwrap().sleeping {
            continue;
        }
        if messages.contains_key(&network_id.id) {
            network_id.last_sent = Instant::now().duration_since(time.startup()).as_nanos();
        }
    }
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
    ass: Res<AssetServer>,
) {
    for event in event_reader.iter() {
        match event {
            ServerEvent::ClientConnected(id, _) => {
                let ship_type = SHIP_TYPES["TEST_SHIP"];

                let ship_gltf_handle = ass.load(
                    Path::new("../../shared/assets/ships").join(Path::new(ship_type.model_name)),
                );

                println!("Player {} connected.", id);
                // Spawn player cube
                let player_entity = commands
                    .spawn(SpatialBundle {
                        transform: Transform {
                            translation: vec3(0.0, 5.0, 0.0),
                            rotation: Quat::from_rotation_x(0.5),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .insert(ShipModelLoadHandle(ship_gltf_handle))
                    .insert(PlayerInput::default())
                    .insert(NetworkedId {
                        id: *id,
                        last_sent: 0,
                    })
                    .insert(ColliderMassProperties::Density(3.0))
                    .insert(Player { team: Team::Red })
                    .insert(Collider::cuboid(2.0, 1.0, 12.0))
                    .insert(CollisionGroups::new(Group::GROUP_1, Group::GROUP_1))
                    .insert(RigidBody::Dynamic)
                    // .insert(LockedAxes::ROTATION_LOCKED_Z)
                    .insert(GravityScale(0.0))
                    .insert(ExternalImpulse::default())
                    .insert(Damping {
                        linear_damping: 0.5,
                        angular_damping: 1.0,
                    })
                    .insert(PbrBundle { ..default() })
                    .id();

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

fn on_client_model_loaded(
    mut commands: Commands,
    query: Query<(Entity, &ShipModelLoadHandle)>,
    assets_gltf: Res<Assets<Gltf>>,
    assets_gltfnode: Res<Assets<GltfNode>>,
) {
    for (entity, handle) in query.iter() {
        if let Some(gltf) = assets_gltf.get(&handle.0) {
            println!("Loaded GLTF, spawning model and turrets");
            // spawn the first scene in the file
            let model = commands
                .spawn(SceneBundle {
                    scene: gltf.scenes[0].clone(),
                    ..Default::default()
                })
                .id();
            let mut turrets: Vec<Entity> = vec![];

            for node_name in gltf.named_nodes.keys().into_iter() {
                if node_name.contains("turret_pad_large") {
                    if let Some(node) = assets_gltfnode.get(&gltf.named_nodes[node_name]) {
                        println!("turret transform: {}", node.transform.translation);
                        let thruster = commands
                            .spawn((
                                TransformBundle::from(node.transform),
                                TurretOwner::new(entity),
                                Turret {
                                    fire_rate: 1.0 / 10.,
                                    ..default()
                                },
                            ))
                            .with_children(|parent| {
                                parent.spawn((TransformBundle::default(), Barrel {}));
                            })
                            .id();
                        turrets.push(thruster);
                    }
                }
            }

            commands
                .entity(entity)
                .push_children(&[model])
                .push_children(&turrets)
                .remove::<ShipModelLoadHandle>();
        }
    }
}
