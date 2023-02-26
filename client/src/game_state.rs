use bevy::{
    ecs::schedule::ShouldRun,
    prelude::{Res, Resource},
};

#[derive(Resource)]
pub struct ClientGameState {
    pub is_paused: bool,
}

pub fn run_if_not_paused(ctx: Res<ClientGameState>) -> ShouldRun {
    match ctx.is_paused {
        true => ShouldRun::No,
        false => ShouldRun::Yes,
    }
}
