use bevy::{
    prelude::{
        default, info, shape, Assets, Color, Commands, Component, EventReader, Mesh, PbrBundle,
        Query, Res, ResMut, StandardMaterial,
    },
    time::Time,
};
use naia_bevy_client::{
    events::{MessageEvent, SpawnEntityEvent, UpdateComponentEvent},
    Client, CommandsExt,
};
use naia_shared::{sequence_greater_than, Tick};

use crate::{
    client::global::OwnedEntity,
    networking::{
        behavior::process_command,
        channels::Channels,
        protocol::{NetworkPosition, Protocol, ProtocolKind},
    },
};

use super::global::ClientGlobal;

pub fn spawn_entity_event(mut event_reader: EventReader<SpawnEntityEvent>, mut commands: Commands) {
    for event in event_reader.iter() {
        match event {
            SpawnEntityEvent(_entity) => {
                commands.entity(*_entity).insert(ClientSide {});
                info!("spawned entity");
            }
        }
    }
}

pub fn update_component_event(
    mut event_reader: EventReader<UpdateComponentEvent<ProtocolKind>>,
    mut global: ResMut<ClientGlobal>,
    time: Res<Time>,
    mut position_query: Query<&mut NetworkPosition>,
) {
    if let Some(owned_entity) = &global.owned_entity {
        let mut latest_tick: Option<Tick> = None;
        let server_entity = owned_entity.confirmed;
        let client_entity = owned_entity.predicted;

        for event in event_reader.iter() {
            let UpdateComponentEvent(server_tick, updated_entity, _) = event;

            // If entity is owned
            if *updated_entity == server_entity {
                if let Some(last_tick) = &mut latest_tick {
                    if sequence_greater_than(*server_tick, *last_tick) {
                        *last_tick = *server_tick;
                    }
                } else {
                    latest_tick = Some(*server_tick);
                }
            }
        }

        if let Some(server_tick) = latest_tick {
            if let Ok([server_position, mut client_position]) =
                position_query.get_many_mut([server_entity, client_entity])
            {
                let replay_commands = global.command_history.replays(&server_tick);

                // set to authoritative state
                client_position.x.mirror(&server_position.x);
                client_position.y.mirror(&server_position.y);

                // Replay all stored commands
                for (_command_tick, command) in replay_commands {
                    process_command(&command, &mut client_position, &time);
                }
            }
        }
    }
}

pub fn receive_message_event(
    mut event_reader: EventReader<MessageEvent<Protocol, Channels>>,
    mut local: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut global: ResMut<ClientGlobal>,
    client: Client<Protocol, Channels>,
) {
    for event in event_reader.iter() {
        if let MessageEvent(Channels::EntityAssignment, Protocol::EntityAssignment(message)) = event
        {
            let assign = *message.assign;

            let entity = message.entity.get(&client).unwrap();
            if assign {
                info!("gave ownership of entity");

                let prediction_entity =
                    CommandsExt::<Protocol>::duplicate_entity(&mut local, entity)
                        .insert(PbrBundle {
                            mesh: meshes.add(shape::Cube { size: 1. }.into()),
                            material: materials.add(Color::GREEN.into()),
                            ..default()
                        })
                        .insert(ClientSide {})
                        .id();

                global.owned_entity = Some(OwnedEntity::new(entity, prediction_entity));
            } else {
                let mut disowned: bool = false;
                if let Some(owned_entity) = &global.owned_entity {
                    if owned_entity.confirmed == entity {
                        local.entity(owned_entity.predicted).despawn();
                        disowned = true;
                    }
                }
                if disowned {
                    info!("removed ownership of entity");
                    global.owned_entity = None;
                }
            }
        }
    }
}

#[derive(Component)]
pub struct ClientSide();
