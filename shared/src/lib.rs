use std::collections::HashMap;

use bevy_ecs::{
    prelude::{Component, Entity},
    system::Resource,
};
use bevy_math::{Quat, Vec3};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Component, Resource)]
pub struct PlayerInput {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub primary_fire: bool,
}

pub const PROTOCOL_ID: u64 = 7;

#[derive(Debug, Default, Resource)]
pub struct Lobby {
    pub players: HashMap<u64, Entity>,
}

#[derive(Debug, Serialize, Deserialize, Component)]
pub enum ServerMessages {
    PlayerConnected { id: u64 },
    PlayerDisconnected { id: u64 },
    BulletSpawned { position: Vec3, rotation: Quat },
}
