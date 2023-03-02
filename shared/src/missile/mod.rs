use std::collections::HashMap;

use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::utils::Instant;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_renet::renet::{DefaultChannel, RenetServer};

use crate::player::player_input::PlayerInput;
use crate::turret::bullet::{Bullet, BulletBundle};
use crate::util::shared_asset;
use crate::{run_if_server, NetworkedId, ServerMessages};

pub struct MissilePlugin;

impl Plugin for MissilePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(RonAssetPlugin::<MissileConfig>::new(&["ron"]))
            .add_startup_system(load_missile_config)
            .add_system(handle_missile_stat_load)
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(run_if_server)
                    .with_system(fire_missile_server),
            )
            .insert_resource(MissileConfigs {
                configs: HashMap::new(),
            });
    }
}

fn fire_missile_server(
    q_input: Query<(&PlayerInput, &Transform)>,
    mut commands: Commands,
    missile_configs: Res<MissileConfigs>,
    time: Res<Time>,
    mut server: ResMut<RenetServer>,
) {
    for (input, transform) in q_input.iter() {
        if input.ability_slot_1 {
            let now = Instant::now();
            let since_start = now.duration_since(time.startup());
            let id = since_start.as_nanos();

            let config = &missile_configs.configs.get("missile");

            match config {
                Some(config) => {
                    println!("lifetime: {} speed: {}", config.lifetime, config.speed);
                    let bullet_transform = TransformBundle::from_transform(*transform);
                    let bullet = Bullet {
                        speed: config.speed,
                        lifetime: time.elapsed_seconds() + config.lifetime,
                    };
                    commands
                        .spawn(bullet_transform)
                        .insert(BulletBundle::new(bullet))
                        .insert(NetworkedId {
                            id: id.try_into().unwrap(),
                            last_sent: 0,
                        });

                    let message = bincode::serialize(&ServerMessages::BulletSpawned {
                        id: id.try_into().unwrap(),
                        position: transform.translation,
                        rotation: transform.rotation,
                    })
                    .unwrap();

                    server.broadcast_message(DefaultChannel::Reliable, message);
                }
                None => (),
            }
        }
    }
}

fn load_missile_config(mut commands: Commands, asset_server: Res<AssetServer>) {
    let missile = asset_server.load(shared_asset("missiles/missile.ron"));
    let missile2 = asset_server.load(shared_asset("missiles/missile2.ron"));
    let handle = MissileHandle(vec![missile, missile2]);
    commands.insert_resource(handle);
}

fn handle_missile_stat_load(
    missile_handles: Res<MissileHandle>,
    mut missile_configs: ResMut<MissileConfigs>,
    mut missile_config_assets: ResMut<Assets<MissileConfig>>,
) {
    for handle in missile_handles.0.clone().into_iter() {
        if let Some(config) = missile_config_assets.remove(handle.id()) {
            missile_configs.configs.insert(config.id.clone(), config);
        }
    }
}

#[derive(serde::Deserialize, TypeUuid)]
#[uuid = "f134d66a-db19-4b3b-8be2-68f245dccc3a"]
pub struct MissileConfig {
    pub id: String,
    pub speed: f32,
    pub lifetime: f32,
}

#[derive(Resource)]
struct MissileHandle(Vec<Handle<MissileConfig>>);

#[derive(Default, Resource)]
pub struct MissileConfigs {
    pub configs: HashMap<String, MissileConfig>,
}
