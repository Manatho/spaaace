use bevy::{
    gltf::Gltf,
    prelude::{
        App, AssetServer, Commands, Component, DespawnRecursiveExt, EventReader, Handle, Plugin,
        Res, ResMut, SpatialBundle,
    },
    utils::default,
};
use bevy_renet::renet::RenetClient;
use spaaaace_shared::{Lobby, ServerMessages};

use crate::camera::OrbitCameraTarget;

pub struct ClientPlayerPlugin;

impl Plugin for ClientPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(on_client_connected);
        app.add_system(on_client_disconnected);
    }
}

#[derive(Component)]
pub struct ShipModelLoadHandle(pub Handle<Gltf>);

fn on_client_connected(
    mut commands: Commands,
    mut lobby: ResMut<Lobby>,
    mut event_reader: EventReader<ServerMessages>,
    client: ResMut<RenetClient>,
    ass: Res<AssetServer>,
) {
    for event in event_reader.iter() {
        match event {
            ServerMessages::PlayerConnected { id } => {
                println!("Player {} connected.", id);

                let my_gltf = ass.load("../../shared/assets/ships/test_ship/test_ship.gltf");
                let mut cmd =
                    commands.spawn((SpatialBundle { ..default() }, ShipModelLoadHandle(my_gltf)));

                if *id == client.client_id() {
                    cmd.insert(OrbitCameraTarget {});
                }

                let player_entity = cmd.id();

                lobby.players.insert(*id, player_entity);
            }

            _ => {}
        }
    }
}

fn on_client_disconnected(
    mut commands: Commands,
    mut lobby: ResMut<Lobby>,
    mut event_reader: EventReader<ServerMessages>,
) {
    for event in event_reader.iter() {
        match event {
            ServerMessages::PlayerDisconnected { id } => {
                println!("Player {} disconnected.", id);
                if let Some(player_entity) = lobby.players.remove(&id) {
                    commands.entity(player_entity).despawn_recursive();
                }
            }
            _ => {}
        }
    }
}
