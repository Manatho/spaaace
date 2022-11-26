use bevy::prelude::{App, MaterialPlugin, Plugin};

use self::capture_point::{capture_system, ForceFieldMaterial};

pub mod capture_point;

pub struct CapturePointPlugin;

impl Plugin for CapturePointPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(capture_system);
        app.add_plugin(MaterialPlugin::<ForceFieldMaterial>::default());
    }
}
