use bevy::prelude::{Component, Resource, Vec3};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Component, Resource, Clone, Copy)]
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
    pub ability_slot_1: bool,
    pub ability_slot_2: bool,
    pub ability_slot_3: bool,
    pub ability_slot_4: bool,
    pub ability_slot_5: bool,
    pub ability_slot_6: bool,
    pub ability_slot_7: bool,
    pub ability_slot_8: bool,
    pub ability_slot_9: bool,
    pub aim_point: Vec3,
    pub target_network_id: u64,
}
