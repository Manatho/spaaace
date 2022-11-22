use bevy::prelude::{App, Plugin};

use self::space_ship::sometimes_move;

pub mod space_ship;

pub struct SpaceShipPlugin;

impl Plugin for SpaceShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(sometimes_move);
    }
}
