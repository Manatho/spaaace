use bevy::{
    math::vec3,
    prelude::{
        App, Color, Component, CoreStage, Plugin, Quat, Query, ResMut, SystemStage, Transform, Vec3,
    },
    time::FixedTimestep,
    utils::HashMap,
};
use bevy_mod_gizmos::{draw_gizmo, Gizmo};
use bevy_rapier3d::prelude::ExternalForce;

use bevy_renet::renet::{DefaultChannel, RenetServer};
use spaaaace_shared::{team::team_enum::Team, PlayerInput, TranslationRotation, SERVER_TICKRATE};

use crate::FixedUpdateStage;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update_players_system)
            .add_system(draw_player_gizmos)
            .add_stage_after(
                CoreStage::Update,
                FixedUpdateStage,
                SystemStage::parallel()
                    .with_run_criteria(FixedTimestep::step(1.0 / (SERVER_TICKRATE as f64)))
                    .with_system(server_sync_players),
            );
    }
}

const PLAYER_MOVE_SPEED: f32 = 2.0;

#[derive(Component, Clone, Hash, PartialEq, Eq)]
pub struct Player {
    pub id: u64,
    pub team: Team,
}

fn update_players_system(mut query: Query<(&mut ExternalForce, &Transform, &PlayerInput)>) {
    for (mut rigidbody, transform, input) in query.iter_mut() {
        let rotation = (input.rotate_right as i8 - input.rotate_left as i8) as f32;
        let thrust_longitudal = (input.thrust_forward as i8 - input.thrust_reverse as i8) as f32;
        let thrust_lateral = (input.thrust_left as i8 - input.thrust_right as i8) as f32;
        let thrust_vertical = (input.thrust_up as i8 - input.thrust_down as i8) as f32;

        let forward = transform.forward();
        let projected_forward = (forward - Vec3::new(0.0, forward.y, 0.0)).normalize();
        let rotated_forward =
            (Quat::from_axis_angle(transform.left(), -0.6 * thrust_vertical)) * projected_forward;

        let left = transform.left();
        let projected_left = (left - Vec3::new(0.0, left.y, 0.0)).normalize();

        let longitudal_force = thrust_longitudal * PLAYER_MOVE_SPEED * 20.0 * projected_forward;
        let lateral_force = thrust_lateral * PLAYER_MOVE_SPEED * 5.0 * projected_left;
        let vertical_force = thrust_vertical * PLAYER_MOVE_SPEED * 10.0 * Vec3::Y;

        draw_gizmo(Gizmo::cubiod(
            transform.translation + rotated_forward * 2.0,
            vec3(0.3, 0.3, 0.3),
            Color::PURPLE,
        ));

        draw_gizmo(Gizmo::cubiod(
            transform.translation + transform.forward() * 2.5,
            vec3(0.3, 0.3, 0.3),
            Color::GREEN,
        ));

        rigidbody.force = longitudal_force + lateral_force + vertical_force;
        rigidbody.torque = rotation * Vec3::NEG_Y * PLAYER_MOVE_SPEED * 2.0;

        {
            let (axis, angle) =
                Quat::from_rotation_arc(transform.forward(), rotated_forward).to_axis_angle();
            rigidbody.torque += axis.normalize_or_zero() * angle;
        }

        {
            let (axis, angle) = Quat::from_rotation_arc(transform.up(), Vec3::Y).to_axis_angle();
            rigidbody.torque += axis.normalize_or_zero() * angle * 10.0;
        }
    }
}

fn server_sync_players(mut server: ResMut<RenetServer>, query: Query<(&Transform, &Player)>) {
    let mut players: HashMap<u64, TranslationRotation> = HashMap::new();
    for (transform, player) in query.iter() {
        players.insert(
            player.id,
            TranslationRotation {
                translation: transform.translation,
                rotation: transform.rotation,
            },
        );
    }

    let sync_message = bincode::serialize(&players).unwrap();
    server.broadcast_message(DefaultChannel::Unreliable, sync_message);
}

fn draw_player_gizmos(query: Query<(&Player, &Transform)>) {
    for (_, transform) in query.iter() {
        draw_gizmo(Gizmo::sphere(transform.translation, 1.0, Color::RED))
    }
}
