use bevy::{
    math::vec3,
    prelude::{App, Commands, Plugin, Transform},
    transform::TransformBundle,
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
            radius: 50.,
            progress: 0.0,
            owner: Team::Neutral,
        });
    commands
        .spawn(TransformBundle {
            local: Transform {
                translation: vec3(50.0, 50.0, 100.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(CaptureSphere {
            radius: 50.,
            progress: 0.0,
            owner: Team::Neutral,
        });
    commands
        .spawn(TransformBundle {
            local: Transform {
                translation: vec3(-100.0, 10.0, 200.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(CaptureSphere {
            radius: 50.,
            progress: 0.0,
            owner: Team::Neutral,
        });
}
