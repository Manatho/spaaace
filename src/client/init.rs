use bevy::{ecs::system::Commands, log::info, prelude::Camera2dBundle};

use naia_bevy_client::Client;

use crate::networking::{
    channels::Channels,
    protocol::{Auth, Protocol},
};

pub fn client_init(mut commands: Commands, mut client: Client<Protocol, Channels>) {
    info!("Naia Bevy Client Demo started");

    client.auth(Auth::new("charlie", "12345"));
    client.connect("http://127.0.0.1:14191");
}
