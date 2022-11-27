use bevy::prelude::{App, Plugin, IntoSystemDescriptor};
use naia_bevy_client::{ClientConfig, Plugin as NaiaClientPlugin, Stage as ClientStage};

use crate::{networking::{channels::Channels, protocol::Protocol, shared::shared_config}, server::server_init};

use self::init::client_init;

pub mod events;
pub mod init;
pub mod resources;

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(NaiaClientPlugin::<Protocol, Channels>::new(
            ClientConfig::default(),
            shared_config(),
        ))
        .add_startup_system(client_init.after(server_init))
        .add_system_to_stage(ClientStage::ReceiveEvents, events::spawn_entity_event)
        .add_system_to_stage(ClientStage::ReceiveEvents, events::update_component_event);
    }
}
