pub mod events;
mod init;
mod resources;
mod tick;

use bevy::prelude::{App, Plugin};
pub use init::server_init;
pub use tick::tick;

use crate::networking::{channels::Channels, protocol::Protocol, shared::shared_config};
use naia_bevy_server::{Plugin as NaiaServerPlugin, ServerConfig, Stage as ServerStage};

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(server_init)
            .add_plugin(NaiaServerPlugin::<Protocol, Channels>::new(
                ServerConfig::default(),
                shared_config(),
            ))
            .add_system_to_stage(ServerStage::ReceiveEvents, events::authorization_event)
            .add_system_to_stage(ServerStage::ReceiveEvents, events::connection_event)
            .add_system_to_stage(ServerStage::ReceiveEvents, events::disconnection_event)
            .add_system_to_stage(ServerStage::ReceiveEvents, events::receive_message_event)
            .add_system_to_stage(ServerStage::Tick, tick);
    }
}
