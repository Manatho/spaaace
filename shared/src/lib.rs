pub mod player;
pub mod ships;
pub mod team;
pub mod util;
pub mod weapons;
pub mod health;
pub mod asteroid;
pub mod targeting;
pub mod cooldown;
pub mod lifetime;
pub mod missile;

use std::collections::HashMap;

use bevy::{
    ecs::schedule::ShouldRun,
    prelude::{Component, Entity, Quat, Res, Resource, Vec3},
};
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
    AsteroidSpawned {
        id: u64,
        position: Vec3,
        scale: Vec3,
        rotation: Quat,
    },
    BalloonSpawned {
        id: u64,
        position: Vec3,
        scale: Vec3,
        rotation: Quat,
    },
}

#[derive(Component)]
pub struct NetworkedId {
    pub id: u64,
    pub last_sent: u128,
}

#[derive(Resource, Debug, Clone)]
#[repr(transparent)]
pub struct NetworkIdProvider(u64);

impl NetworkIdProvider {
    pub fn new() -> Self {
        Self(0)
    }

    /// Creates a new, unique [`NetworkId`].
    pub fn new_id(&mut self) -> NetworkedId {
        let id = NetworkedId {
            id: self.0,
            last_sent: 0,
        };
        self.0 = self
            .0
            .checked_add(1)
            .expect("NetworkId has overflowed u32::MAX.");
        id
    }
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

#[derive(Resource)]
pub struct NetworkContext {
    pub is_server: bool,
}

pub fn run_if_server(ctx: Res<NetworkContext>) -> ShouldRun {
    match ctx.is_server {
        true => ShouldRun::Yes,
        false => ShouldRun::No,
    }
}

pub fn run_if_client(ctx: Res<NetworkContext>) -> ShouldRun {
    match ctx.is_server {
        true => ShouldRun::No,
        false => ShouldRun::Yes,
    }
}
