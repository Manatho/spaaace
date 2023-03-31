use bevy::{
    prelude::{Bundle, Component, PbrBundle, Query, Res, ResMut, Transform},
    time::Time,
    utils::{HashMap, HashSet},
};
use bevy_renet::renet::{DefaultChannel, RenetServer};
use spaaaace_shared::{team::team_enum::Team, ServerMessages};

use crate::Player;

#[derive(Component, Clone)]
pub struct CaptureSphere {
    pub id: u64,
    pub radius: f32,
    pub progress: f32,
    pub owner: Team,
    pub attackers: HashSet<Player>,
}

#[derive(Bundle)]
pub struct CapturePoint {
    pub capture: CaptureSphere,
    pub pbr_bundle: PbrBundle,
}

pub fn capture_arena(
    mut query_capture_spheres: Query<(&Transform, &mut CaptureSphere)>,
    query_space_ship: Query<(&Transform, &Player)>,
) {
    for (capture_transform, mut capture_sphere) in query_capture_spheres.iter_mut() {
        capture_sphere.attackers.clear();
        for (ship_transform, player) in query_space_ship.iter() {
            let distance = ship_transform
                .translation
                .distance(capture_transform.translation);

            if capture_sphere.radius > distance {
                capture_sphere.attackers.insert(player.clone());
            }
        }
    }
}

pub fn capture_progress(
    mut query_capture_spheres: Query<&mut CaptureSphere>,
    time: Res<Time>,
    mut server: ResMut<RenetServer>,
) {
    for mut capture_sphere in query_capture_spheres.iter_mut() {
        let old_progress = capture_sphere.progress;
        let mut attackers_by_team = HashMap::<Team, u8>::new();

        for element in capture_sphere.attackers.clone().into_iter() {
            let team = element.team;

            let current_count = attackers_by_team.get(&team).unwrap_or(&0);

            attackers_by_team.insert(team.clone(), *current_count);
        }

        let superior_team = attackers_by_team
            .iter()
            .max_by(|(_, count1), (_, count2)| count1.cmp(count2));
        let capture_rate = 0.1;

        if let Some((team, _)) = superior_team {
            if capture_sphere.owner == Team::Neutral {
                capture_sphere.progress =
                    (capture_sphere.progress + capture_rate * time.delta_seconds()).clamp(0.0, 1.0);

                if capture_sphere.progress == 1.0 {
                    capture_sphere.owner = team.clone();
                }
            } else if &capture_sphere.owner != team {
                capture_sphere.progress =
                    (capture_sphere.progress - capture_rate * time.delta_seconds()).clamp(0.0, 1.0);

                if capture_sphere.progress == 0.0 {
                    capture_sphere.owner = Team::Neutral;
                }
            }
        } else if capture_sphere.progress < 1.0 && capture_sphere.owner != Team::Neutral {
            capture_sphere.progress =
                (capture_sphere.progress + capture_rate * time.delta_seconds()).clamp(0.0, 1.0);
        } else if capture_sphere.progress > 0.0 && capture_sphere.owner == Team::Neutral {
            capture_sphere.progress =
                (capture_sphere.progress - capture_rate * time.delta_seconds()).clamp(0.0, 1.0);
        }

        if old_progress != capture_sphere.progress {
            let message = bincode::serialize(&ServerMessages::CapturePointUpdate {
                id: capture_sphere.id,
                owner: capture_sphere.owner.clone(),
                attacker: capture_sphere.owner.clone(),
                progress: capture_sphere.progress,
            })
            .unwrap();
            server.broadcast_message(DefaultChannel::Reliable, message);
        }
    }
}
