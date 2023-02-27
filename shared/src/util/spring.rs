use bevy::{
    prelude::{App, Component, Plugin, Query, Res, Transform, Vec3},
    time::Time,
};

pub struct SpringPlugin;

impl Plugin for SpringPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(local_spring_system);
    }
}

#[derive(Component)]
pub struct LocalSpring {
    pub resting_transform: Transform,
}

#[derive(Component)]
pub struct SpringVelocity {
    pub value: Vec3,
}

pub fn local_spring_system(
    mut spring_query: Query<(&mut Transform, &LocalSpring, &mut SpringVelocity)>,
    time: Res<Time>,
) {
    for (mut transform, local_spring, mut spring_velocity) in spring_query.iter_mut() {
        let rest_diff = local_spring.resting_transform.translation - transform.translation;
        spring_velocity.value += rest_diff;
        spring_velocity.value *= 0.9;
        transform.translation += spring_velocity.value * time.delta_seconds();
    }
}
