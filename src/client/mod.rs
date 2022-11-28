use std::f32::consts::PI;

use bevy::prelude::{
    default, shape, App, Assets, Camera3dBundle, Color, Commands, DirectionalLight,
    DirectionalLightBundle, IntoSystemDescriptor, Mesh, OrthographicProjection, PbrBundle, Plugin,
    Quat, ResMut, StandardMaterial, Transform, Vec3,
};
use naia_bevy_client::{ClientConfig, Plugin as NaiaClientPlugin, Stage as ClientStage};

use crate::{
    networking::{channels::Channels, protocol::Protocol, shared::shared_config},
    server::server_init,
};

use self::{global::ClientGlobal, init::client_init};

pub mod events;
pub mod global;
pub mod init;
pub mod input;
pub mod sync;
pub mod tick;

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(NaiaClientPlugin::<Protocol, Channels>::new(
            ClientConfig::default(),
            shared_config(),
        ))
        // .add_startup_system(client_init.after(server_init))
        .add_system_to_stage(ClientStage::ReceiveEvents, events::spawn_entity_event)
        .add_system_to_stage(ClientStage::ReceiveEvents, events::receive_message_event)
        .add_system_to_stage(ClientStage::ReceiveEvents, events::update_component_event)
        .add_system_to_stage(ClientStage::Frame, input::input)
        .add_system_to_stage(ClientStage::PostFrame, sync::sync)
        .add_system_to_stage(ClientStage::Tick, tick::tick)
        .init_resource::<ClientGlobal>();
    }
}
