use bevy::prelude::{App, Plugin};

pub mod projectile;

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(projectile::move_projectile_system);
    }
}
