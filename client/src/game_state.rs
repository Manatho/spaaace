use bevy::prelude::{Res, Resource};

#[derive(Resource)]
pub struct ClientGameState {
    pub is_paused: bool,
    pub is_focused: bool,
}

pub fn run_if_not_paused(ctx: Res<ClientGameState>) -> bool {
    return !ctx.is_paused;
}
