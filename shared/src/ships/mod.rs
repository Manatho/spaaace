use bevy::{
    gltf::{Gltf, GltfNode},
    prelude::{
        default, App, AssetServer, Assets, BuildChildren, Commands, Component, Entity, Handle,
        Plugin, Query, Res, SpatialBundle, Transform,
    },
    scene::SceneBundle,
};
use phf::phf_map;

use crate::turret::model_load_handlers::{spawn_turret, TurretModelLoadHandle};

#[derive(Clone, Copy)]
pub struct ShipType {
    // model_name is relative to the assets/ships/ folder
    pub model_name: &'static str,
    pub forward_thrust_force: f32,
    pub backward_thrust_force: f32,
    pub lateral_thrust_force: f32,
}

pub static SHIP_TYPES: phf::Map<&'static str, ShipType> = phf_map! {
    "TEST_SHIP" => ShipType{
        model_name: "test_ship/test_ship.gltf",
        forward_thrust_force: 2000.,
        backward_thrust_force: 2000.,
        lateral_thrust_force: 2000.,
    },
};

pub struct ShipsPlugin;

impl Plugin for ShipsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(handle_ship_model_load);
    }
}

#[derive(Component)]
pub struct ShipModelLoadHandle(pub Handle<Gltf>);

pub fn handle_ship_model_load(
    mut commands: Commands,
    query: Query<(Entity, &ShipModelLoadHandle)>,
    assets_gltf: Res<Assets<Gltf>>,
    assets_gltfnode: Res<Assets<GltfNode>>,
    ass: Res<AssetServer>,
    // mut effects: ResMut<Assets<EffectAsset>>,
) {
    for (entity, handle) in query.iter() {
        if let Some(gltf) = assets_gltf.get(&handle.0) {
            println!("Loaded GLTF, spawning...");
            // spawn the first scene in the file
            let model = commands
                .spawn(SceneBundle {
                    scene: gltf.scenes[0].clone(),
                    ..Default::default()
                })
                .id();
            let mut turrets: Vec<Entity> = vec![];

            for node_name in gltf.named_nodes.keys().into_iter() {
                if node_name.contains("turret_pad_large") {
                    if let Some(node) = assets_gltfnode.get(&gltf.named_nodes[node_name]) {
                        let turret = spawn_turret(&mut commands, &ass, &node.transform, entity);
                        turrets.push(turret);
                    }
                }
            }

            commands
                .entity(entity)
                .push_children(&[model])
                .push_children(&turrets)
                .remove::<ShipModelLoadHandle>();
        }
    }
}
