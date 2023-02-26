use bevy::{
    prelude::{
        App, AssetServer, Assets, Bundle, Commands, Component, Handle, HandleUntyped, Plugin,
        Query, Res, ResMut, Resource, Transform,
    },
    reflect::TypeUuid,
    text::Font,
    time::Time,
};
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_rapier3d::prelude::{ActiveEvents, Collider, Sensor};

use crate::lifetime::LifeTime;

#[derive(Resource)]
struct MissileAssets(Vec<Handle<MissileStats>>);

#[derive(serde::Deserialize, TypeUuid)]
#[uuid = "f134d66a-db19-4b3b-8be2-68f245dccc3a"]
pub struct MissileStats {
    speed: f32,
    lifetime: f32,
}

fn load_missile_assets(mut commands: Commands, server: Res<AssetServer>) {
    let mut handles = Vec::new();
    handles.push(server.load("asd/default.ron"));
    commands.insert_resource(MissileAssets(handles));
}

pub struct MissilePlugin;

impl Plugin for MissilePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(load_missile_assets)
            .add_plugin(RonAssetPlugin::<MissileStats>::new(&["ron"]))
            .add_system(missile_mover)
            .add_system(test);
    }
}
#[derive(Component, Clone, Copy)]
pub struct Missile {
    pub speed: f32,
}

#[derive(Bundle)]
pub struct MissileBundle {
    pub missile: Missile,
    active_events: ActiveEvents,
    collider: Collider,
    sensor: Sensor,
    lifetime: LifeTime,
}

impl MissileBundle {
    pub fn new(stats: MissileStats) -> Self {
        MissileBundle {
            missile: Missile { speed: stats.speed },
            active_events: ActiveEvents::COLLISION_EVENTS,
            collider: Collider::ball(0.5),
            lifetime: LifeTime {
                value: stats.lifetime,
            },
            sensor: Sensor,
        }
    }
}

fn missile_mover(
    mut query: Query<(&mut Transform, &Missile)>, //
    time: Res<Time>,
) {
    for (mut transform, missile) in query.iter_mut() {
        let dir = transform.forward();
        transform.translation += dir * time.delta_seconds() * missile.speed;
    }
}

fn test(
    mut commands: Commands,
    assets_handles: Res<MissileAssets>,
    mut assets: ResMut<Assets<MissileStats>>,
) {
    for asset in assets_handles.0.iter() {
        println!("Got one");

        let x = assets.get(asset);

        for id in assets.ids() {
            println!("hmmmmm {}", id == asset.id())
        }

        if let Some(stats) = x {
            println!("{}", stats.speed);
        } else {
            println!("No stats :(");
        }
    }
}
