use bevy::{
    prelude::{Component, Query, Res, Transform},
    time::Time,
};

#[derive(Component)]
pub struct LerpTransformTarget {
    pub target: Transform,
    pub speed: f32,
}

pub fn lerp_transform_targets(
    mut query: Query<(&mut Transform, &LerpTransformTarget)>,
    time: Res<Time>,
) {
    for (mut t, l) in query.iter_mut() {
        let s = l.speed * time.delta_seconds();
        t.translation = t.translation.lerp(l.target.translation, s);
        t.rotation = t.rotation.lerp(l.target.rotation, s);
        // t.scale = t.scale.lerp(l.target.scale, s); Removed since current scale is not sent
    }
}
