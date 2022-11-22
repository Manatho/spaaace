use bevy::prelude::{App, Plugin};

use self::capture_point::capture_system;

mod capture_point;

pub struct CapturePointPlugin;

impl Plugin for CapturePointPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(capture_system);
    }
}
