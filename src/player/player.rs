use bevy::prelude::Component;

use crate::team::team_enum::Team;

#[derive(Component, Clone, Hash, PartialEq, Eq)]
pub struct Player {
    pub team: Team,
}
