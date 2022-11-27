use bevy::prelude::{App, MaterialPlugin, Plugin};

use self::capture_point::{capture_arena, ForceFieldMaterial, capture_progress};

pub mod capture_point;

pub struct CapturePointPlugin;

impl Plugin for CapturePointPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(capture_arena);
        app.add_system(capture_progress);
        app.add_plugin(MaterialPlugin::<ForceFieldMaterial>::default());
    }
}
