use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy_common_assets::ron::RonAssetPlugin;

use crate::util::shared_asset;

pub struct MissilePlugin;

impl Plugin for MissilePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(RonAssetPlugin::<MissileStats>::new(&["ron"]))
            .add_startup_system(setup)
            .add_system(spawn_level);
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let missile = asset_server.load(shared_asset("missiles/missile.ron"));
    let missile2 = asset_server.load(shared_asset("missiles/missile2.ron"));
    let handle = MissileHandle(vec![missile, missile2]);
    commands.insert_resource(handle);
}

fn spawn_level(
    missile_handles: Res<MissileHandle>,
    mut missilestats: ResMut<Assets<MissileStats>>,
) {
    for handle in missile_handles.0.clone().into_iter() {
        if let Some(stat) = missilestats.remove(handle.id()) {
            println!("{} {}", stat.speed, stat.lifetime);
        }
    }
}

#[derive(serde::Deserialize, TypeUuid)]
#[uuid = "f134d66a-db19-4b3b-8be2-68f245dccc3a"]
pub struct MissileStats {
    pub speed: f32,
    pub lifetime: f32,
}

#[derive(Resource)]
struct MissileHandle(Vec<Handle<MissileStats>>);
