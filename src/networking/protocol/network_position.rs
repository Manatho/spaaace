use bevy::prelude::Component;

use naia_shared::{Property, Replicate};

#[derive(Component, Replicate)]
#[protocol_path = "crate::networking::protocol::Protocol"]
pub struct NetworkPosition {
    pub x: Property<f32>,
    pub y: Property<f32>,
}

impl NetworkPosition {
    pub fn new(x: f32, y: f32) -> Self {
        NetworkPosition::new_complete(x, y)
    }
}