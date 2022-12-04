use bevy::{
    math::vec3,
    prelude::{Query, Res, Transform},
    time::Time,
};

use crate::networking::protocol::NetworkPosition;

pub fn sync(mut query: Query<(&NetworkPosition, &mut Transform)>, time: Res<Time>) {
    for (pos, mut transform) in query.iter_mut() {
        transform.translation = transform.translation.lerp(
            vec3(
                f32::from(*pos.x),
                f32::from(*pos.y),
                transform.translation.z,
            ),
            time.delta_seconds() * 10.,
        )
    }
}
