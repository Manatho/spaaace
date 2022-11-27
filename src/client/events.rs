use bevy::prelude::{EventReader, info};
use naia_bevy_client::events::SpawnEntityEvent;

pub fn spawn_entity_event(mut event_reader: EventReader<SpawnEntityEvent>) {
    for event in event_reader.iter() {
        match event {
            SpawnEntityEvent(_entity) => {
                info!("spawned entity");
            }
        }
    }
}
