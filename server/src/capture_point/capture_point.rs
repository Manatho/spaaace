use bevy::{
    prelude::{Bundle, Component, PbrBundle},
};
use spaaaace_shared::team::team_enum::Team;

#[derive(Component, Clone)]
pub struct CaptureSphere {
    pub radius: f32,
    pub progress: f32,
    pub owner: Team,
}

#[derive(Bundle)]
pub struct CapturePoint {
    pub capture: CaptureSphere,
    pub pbr_bundle: PbrBundle,
}
