use bevy::{
    math::vec3,
    prelude::{App, Color, Commands, EventReader, Plugin, Query, ResMut, Transform},
    transform::TransformBundle,
    utils::HashSet,
};

use bevy_mod_gizmos::{draw_gizmo, Gizmo};
use bevy_renet::renet::{DefaultChannel, RenetServer, ServerEvent};
use spaaaace_shared::{team::team_enum::Team, ServerMessages};

use self::capture_point::{capture_arena, capture_progress, CaptureSphere};

pub mod capture_point;

pub struct CapturePointPlugin;

impl Plugin for CapturePointPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(init)
            .add_system(capture_arena)
            .add_system(on_client_connected)
            .add_system(draw_capture_sphere_gizmos)
            .add_system(capture_progress);
    }
}

fn on_client_connected(
    mut event_reader: EventReader<ServerEvent>,
    mut server: ResMut<RenetServer>,
    capture_point_query: Query<(&Transform, &CaptureSphere)>,
) {
    for event in event_reader.iter() {
        match event {
            ServerEvent::ClientConnected(id, _) => {
                for (&transform, capture_point) in capture_point_query.iter() {
                    let message = bincode::serialize(&ServerMessages::CapturePointSpawned {
                        position: transform.translation,
                        rotation: transform.rotation,
                        id: capture_point.id,
                        owner: capture_point.owner.clone(),
                        progress: capture_point.progress,
                    })
                    .unwrap();
                    server.send_message(*id, DefaultChannel::Reliable, message);
                }
            }

            _ => (),
        }
    }
}

fn draw_capture_sphere_gizmos(cap_query: Query<(&CaptureSphere, &Transform)>) {
    for (_, transform) in cap_query.iter() {
        draw_gizmo(Gizmo::new(transform.translation, 1.0, Color::GREEN))
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
