use bevy::{
    prelude::{App, Commands, Component, Entity, Plugin, Query, Res},
    time::Time,
};

#[derive(Component)]
pub struct Cooldown {
    pub value: f32,
}

pub struct CooldownPlugin;
impl Plugin for CooldownPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(cooldown);
    }
}

fn cooldown(mut query: Query<(Entity, &mut Cooldown)>, time: Res<Time>, mut commands: Commands) {
    for (entity, mut cooldown) in query.iter_mut() {
        cooldown.value = cooldown.value - time.delta_seconds();

        if cooldown.value < 0.0 {
            commands.entity(entity).remove::<Cooldown>();
        }
    }
}
