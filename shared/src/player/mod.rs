use bevy::prelude::Component;

use crate::team::team_enum::Team;

pub mod player_input;

#[derive(Component, Clone, Hash, PartialEq, Eq)]
pub struct Player {
    pub team: Team,
}
