pub mod player;
pub mod team;
pub mod util;

use std::collections::HashMap;

use bevy_ecs::{
    prelude::{Component, Entity},
    system::Resource,
};
use bevy_math::{Quat, Vec3};
use player::player_input::PlayerInput;
use serde::{Deserialize, Serialize};
use team::team_enum::Team;

#[derive(Debug, Serialize, Deserialize, Component, Clone)]
pub enum ClientMessages {
    PlayerInput { input: PlayerInput },
    Command { command: String },
}

pub const PROTOCOL_ID: u64 = 7;

#[derive(Debug, Default, Resource)]
pub struct Lobby {
    pub networked_entities: HashMap<u64, Entity>,
    pub players: HashMap<u64, Entity>,
    pub capture_points: HashMap<u64, Entity>,
}

#[derive(Debug, Serialize, Deserialize, Component)]
pub enum ServerMessages {
    PlayerConnected {
        id: u64,
    },
    PlayerDisconnected {
        id: u64,
    },
    BulletSpawned {
        id: u64,
        position: Vec3,
        rotation: Quat,
    },
    EntityDespawn {
        id: u64,
    },
    CapturePointSpawned {
        id: u64,
        owner: Team,
        progress: f32,
        position: Vec3,
        rotation: Quat,
    },
    CapturePointUpdate {
        id: u64,
        owner: Team,
        attacker: Team,
        progress: f32,
    },
}

#[derive(Component)]
pub struct NetworkedId {
    pub id: u64,
    pub last_sent: u128
}

#[derive(Component)]
pub struct NetworkedTransform {
    pub send_rate: f32,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TranslationRotation {
    pub translation: Vec3,
    pub rotation: Quat,
}

pub const SERVER_TICKRATE: f32 = 10.0;
