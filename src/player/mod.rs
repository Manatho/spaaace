use bevy::prelude::{App, Input, KeyCode, Plugin, Query, Res};

use crate::team::team_enum::Team;

use self::player::Player;

pub mod player;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(temporary_team_swap);
    }
}

fn temporary_team_swap(mut query: Query<&mut Player>, keyboard: Res<Input<KeyCode>>) {
    for mut player in query.iter_mut() {
        if keyboard.just_released(KeyCode::Tab) {
            if player.team == Team::Blue {
                player.team = Team::Red
            } else {
                player.team = Team::Blue
            }
        }
    }
}
