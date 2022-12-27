use bevy::{
    math::vec3,
    prelude::{App, Commands, Plugin, Transform},
    transform::TransformBundle,
    utils::HashSet,
};

use spaaaace_shared::team::team_enum::Team;

use self::capture_point::{CaptureSphere, capture_arena, capture_progress};

pub mod capture_point;

pub struct CapturePointPlugin;

impl Plugin for CapturePointPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(init)
            .add_system(capture_arena)
            .add_system(capture_progress);
    }
}

fn init(mut commands: Commands) {
    commands
        .spawn(TransformBundle {
            local: Transform {
                translation: vec3(0.0, 100.0, 100.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(CaptureSphere {
            radius: 50.,
            progress: 0.0,
            owner: Team::Neutral,
            attackers: HashSet::default(),
        });
    commands
        .spawn(TransformBundle {
            local: Transform {
                translation: vec3(150.0, 50.0, 100.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(CaptureSphere {
            radius: 50.,
            progress: 0.0,
            owner: Team::Neutral,
            attackers: HashSet::default(),
        });
    commands
        .spawn(TransformBundle {
            local: Transform {
                translation: vec3(200.0, 10.0, 200.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(CaptureSphere {
            radius: 50.,
            progress: 0.0,
            owner: Team::Neutral,
            attackers: HashSet::default(),
        });
}
