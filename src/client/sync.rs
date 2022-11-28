use bevy::prelude::{Query, Transform};

use crate::networking::protocol::NetworkPosition;

pub fn sync(mut query: Query<(&NetworkPosition, &mut Transform)>) {
  for (pos, mut transform) in query.iter_mut() {
      transform.translation.x = f32::from(*pos.x);
      transform.translation.y = f32::from(*pos.y) * -1.0;
  }
}