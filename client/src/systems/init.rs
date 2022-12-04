use bevy::{ecs::system::Commands, log::info, prelude::Camera2dBundle};

use naia_bevy_client::Client;

use spaaaace_shared::{
    protocol::{Auth, Protocol},
    Channels,
};

use crate::resources::Global;

pub fn init(mut commands: Commands, mut client: Client<Protocol, Channels>) {
    info!("Naia Bevy Client Demo started");

    client.auth(Auth::new("charlie", "12345"));
    client.connect("http://127.0.0.1:14191");

    // Setup Camera
    commands.spawn(Camera2dBundle::default());

    // Setup Colors
    commands.init_resource::<Global>();
}
