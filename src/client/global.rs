use bevy::prelude::{Entity, Resource};
use naia_bevy_client::CommandHistory;

use crate::networking::protocol::KeyCommand;

pub struct OwnedEntity {
  pub confirmed: Entity,
  pub predicted: Entity,
}

impl OwnedEntity {
  pub fn new(confirmed_entity: Entity, predicted_entity: Entity) -> Self {
      OwnedEntity {
          confirmed: confirmed_entity,
          predicted: predicted_entity,
      }
  }
}

#[derive(Resource)]
pub struct ClientGlobal {
  pub owned_entity: Option<OwnedEntity>,
  pub queued_command: Option<KeyCommand>,
  pub command_history: CommandHistory<KeyCommand>,
}
