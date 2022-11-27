use bevy::prelude::Component;

use naia_shared::{Property, Replicate};

#[derive(Component, Replicate)]
#[protocol_path = "crate::networking::protocol::Protocol"]
pub struct NetworkPosition {
    pub x: Property<i16>,
    pub y: Property<i16>,
}

impl NetworkPosition {
    pub fn new(x: i16, y: i16) -> Self {
        NetworkPosition::new_complete(x, y)
    }
}