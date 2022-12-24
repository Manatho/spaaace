use bevy::{
    prelude::{App, Commands, Plugin, Transform},
    transform::TransformBundle, math::vec3,
};

use spaaaace_shared::team::team_enum::Team;

use self::capture_point::CaptureSphere;

pub mod capture_point;

pub struct CapturePointPlugin;

impl Plugin for CapturePointPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(init);
    }
}

fn init(mut commands: Commands) {
    commands
        .spawn(TransformBundle {
            local: Transform {
                translation: vec3(0.0, 1.0, 1.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(CaptureSphere {
            radius: 3.,
            progress: 0.0,
            owner: Team::Neutral,
        });
}
