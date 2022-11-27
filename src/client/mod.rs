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
pub mod tick;
pub mod sync;

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(NaiaClientPlugin::<Protocol, Channels>::new(
            ClientConfig::default(),
            shared_config(),
        ))
        .add_startup_system(client_init.after(server_init))
        .add_startup_system(setup)
        .add_system_to_stage(ClientStage::ReceiveEvents, events::spawn_entity_event)
        .add_system_to_stage(ClientStage::ReceiveEvents, events::receive_message_event)
        .add_system_to_stage(ClientStage::ReceiveEvents, events::update_component_event)
        .add_system_to_stage(ClientStage::Frame, input::input)
        .add_system_to_stage(ClientStage::PostFrame, sync::sync)
        .add_system_to_stage(ClientStage::Tick, tick::tick)
        .init_resource::<ClientGlobal>();
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 6., 12.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
        ..default()
    });

    const HALF_SIZE: f32 = 10.0;
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            // Configure the projection to better fit the scene
            shadow_projection: OrthographicProjection {
                left: -HALF_SIZE,
                right: HALF_SIZE,
                bottom: -HALF_SIZE,
                top: HALF_SIZE,
                near: -10.0 * HALF_SIZE,
                far: 10.0 * HALF_SIZE,
                ..default()
            },
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
        ..default()
    });
}
