use bevy::{
    prelude::{AlphaMode, Bundle, Component, Material, PbrBundle, Query, Res, Transform, With},
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef},
    time::Time,
    utils::{hashbrown::HashSet, HashMap},
};

use crate::{player::player::Player, ship::space_ship::SpaceShip, team::team_enum::Team};

#[derive(Component, Clone)]
pub struct CaptureSphere {
    pub radius: f32,
    pub progress: f32,
    pub attackers: HashSet<Player>,
    pub owner: Team,
}

#[derive(Bundle)]
pub struct CapturePoint {
    pub capture: CaptureSphere,
    pub pbr_bundle: PbrBundle,
}

pub fn capture_arena(
    mut query_capture_spheres: Query<(&Transform, &mut CaptureSphere)>,
    query_space_ship: Query<(&Transform, &Player), With<SpaceShip>>,
) {
    for (ship_transform, player) in query_space_ship.iter() {
        for (capture_transform, mut capture_sphere) in query_capture_spheres.iter_mut() {
            let distance = ship_transform
                .translation
                .distance(capture_transform.translation);

            if capture_sphere.radius > distance {
                capture_sphere.attackers.insert(player.clone());
            } else {
                capture_sphere.attackers.remove(player);
            }
        }
    }
}

pub fn capture_progress(mut query_capture_spheres: Query<&mut CaptureSphere>, time: Res<Time>) {
    for mut capture_sphere in query_capture_spheres.iter_mut() {
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
        } else if capture_sphere.owner == Team::Neutral {
            capture_sphere.progress =
                (capture_sphere.progress - capture_rate * time.delta_seconds()).clamp(0.0, 1.0);
        }

        /* println!("{} {}", capture_sphere.progress, capture_sphere.owner) */
    }
}

// This is the struct that will be passed to your shader
#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "f690fdae-d598-45ab-8225-97e2a3f056e0"]
pub struct ForceFieldMaterial {
    // #[uniform(0)]
    // pub selection: Vec4,
}

impl Material for ForceFieldMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/forcefield.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}
