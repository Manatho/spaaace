use bevy::{
    prelude::{
        default, info, shape, Assets, Color, Commands, EventReader, Mesh, PbrBundle, Query, ResMut,
        StandardMaterial, Transform, Vec2,
    },
    sprite::{Sprite, SpriteBundle},
};
use naia_bevy_client::{
    events::{MessageEvent, SpawnEntityEvent, UpdateComponentEvent},
    Client, CommandsExt,
};

use crate::{
    client::global::OwnedEntity,
    networking::{
        behavior::process_command,
        channels::Channels,
        protocol::{NetworkPosition, Protocol, ProtocolKind},
    },
};

use super::global::ClientGlobal;

pub fn spawn_entity_event(mut event_reader: EventReader<SpawnEntityEvent>) {
    for event in event_reader.iter() {
        match event {
            SpawnEntityEvent(_entity) => {
                info!("spawned entity");
            }
        }
    }
}


pub fn receive_message_event(
    mut event_reader: EventReader<MessageEvent<Protocol, Channels>>,
    mut local: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    /* mut global: ResMut<ClientGlobal>, */
    client: Client<Protocol, Channels>,
) {
    /* for event in event_reader.iter() {
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
    } */
}
