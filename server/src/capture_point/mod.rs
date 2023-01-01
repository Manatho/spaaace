use bevy::{
    math::vec3,
    prelude::{App, Color, Commands, Plugin, Query, Transform},
    transform::TransformBundle,
    utils::HashSet,
};

use bevy_mod_gizmos::{draw_gizmo, Gizmo};
use spaaaace_shared::team::team_enum::Team;

use self::capture_point::{capture_arena, capture_progress, CaptureSphere};

pub mod capture_point;

pub struct CapturePointPlugin;

impl Plugin for CapturePointPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(init)
            .add_system(capture_arena)
            .add_system(draw_capture_sphere_gizmos)
            .add_system(capture_progress);
    }
}

fn draw_capture_sphere_gizmos(cap_query: Query<(&CaptureSphere, &Transform)>) {
    for (_, transform) in cap_query.iter() {
        draw_gizmo(Gizmo::sphere(transform.translation, 1.0, Color::GREEN))
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
            owner: Team::Blue,
            attackers: HashSet::default(),
            id: 1,
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
            id: 2,
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
            owner: Team::Red,
            attackers: HashSet::default(),
            id: 3,
        });
}
