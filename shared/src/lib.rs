pub mod team;

use std::collections::HashMap;

use bevy_ecs::{
    prelude::{Component, Entity},
    system::Resource,
};
use bevy_math::{Quat, Vec3};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Component, Resource)]
pub struct PlayerInput {
    pub thrust_forward: bool,
    pub thrust_reverse: bool,
    pub thrust_left: bool,
    pub thrust_right: bool,
    pub thrust_up: bool,
    pub thrust_down: bool,
    pub rotate_left: bool,
    pub rotate_right: bool,
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

#[derive(Component)]
pub struct NetworkedTransform {
    pub id: u64,
    pub send_rate: f32,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TranslationRotation {
    pub translation: Vec3,
    pub rotation: Quat,
}

pub const SERVER_TICKRATE: f32 = 10.0;
