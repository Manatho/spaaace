use bevy::prelude::Resource;

#[derive(Resource)]
pub struct ClientGameState {
    pub is_paused: bool,
}
