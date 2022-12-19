use bevy::{
    prelude::{App, Component, Plugin, Query, Res, Transform, With},
    time::Time,
};

#[derive(Component)]
pub struct Projectiles;

pub struct ProjectilesPlugin;


impl Plugin for ProjectilesPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(move_projectiles);
    }
}

pub fn move_projectiles(mut query: Query<&mut Transform, With<Projectiles>>, time: Res<Time>) {
    for mut transform in query.iter_mut() {
        let forward = transform.forward();
        transform.translation += forward * time.delta_seconds();
    }
}

