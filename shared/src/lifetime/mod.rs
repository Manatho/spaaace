use bevy::{
    prelude::{
        App, Commands, Component, DespawnRecursiveExt, Entity, EventWriter, Plugin, Query, Res,
    },
    time::Time,
};

#[derive(Component)]
pub struct LifeTime {
    pub value: f32,
}

#[derive(Component)]
pub enum LifeTimeEvent {
    Despawned,
}

pub struct LifeTimePlugin;
impl Plugin for LifeTimePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(lifetime).add_event::<LifeTimeEvent>();
    }
}

fn lifetime(
    mut query: Query<(Entity, &mut LifeTime)>,
    time: Res<Time>,
    mut commands: Commands,
    mut lifetime_event_writer: EventWriter<LifeTimeEvent>,
) {
    for (entity, mut lifetime) in query.iter_mut() {
        lifetime.value = lifetime.value - time.delta_seconds();

        if lifetime.value < 0.0 {
            commands.entity(entity).despawn_recursive();
            lifetime_event_writer.send(LifeTimeEvent::Despawned);
        }
    }
}
