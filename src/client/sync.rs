use bevy::{
    math::vec3,
    prelude::{Query, Res, Transform, With},
    time::Time,
};

use crate::networking::protocol::NetworkPosition;

use super::events::ClientSide;

pub fn sync(mut query: Query<(&NetworkPosition, &mut Transform), With<ClientSide>>, time: Res<Time>) {
    for (pos, mut transform) in query.iter_mut() {
        transform.translation.x = f32::from(*pos.x);
        transform.translation.y = f32::from(*pos.y) * -1.0;

        /*  transform.translation = transform.translation.lerp(
            vec3(
                f32::from(*pos.x),
                f32::from(*pos.y),
                transform.translation.z,
            ),
            1.,
        ) */
    }
}
