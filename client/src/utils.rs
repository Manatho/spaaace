use bevy::{
    gltf::{Gltf, GltfNode},
    prelude::{
        default, AssetServer, Assets, BuildChildren, Commands, Component, Entity, Handle, Name,
        Query, Res, ResMut, SpatialBundle, Transform, Vec2, Vec4,
    },
    scene::SceneBundle,
    time::Time,
};
use bevy_hanabi::{
    BillboardModifier, ColorOverLifetimeModifier, EffectAsset, Gradient, InitLifetimeModifier,
    InitPositionSphereModifier, InitVelocitySphereModifier, ParticleEffectBundle, ShapeDimension,
    SizeOverLifetimeModifier, Spawner,
};
use bevy_scene_hook::{HookedSceneBundle, SceneHook};
use spaaaace_shared::weapons::{Barrel, Turret, TurretOwner};

use crate::player::ShipModelLoadHandle;

#[derive(Component)]
pub struct LerpTransformTarget {
    pub target: Transform,
    pub speed: f32,
}

pub fn lerp_transform_targets(
    mut query: Query<(&mut Transform, &LerpTransformTarget)>,
    time: Res<Time>,
) {
    for (mut t, l) in query.iter_mut() {
        let s = l.speed * time.delta_seconds();
        t.translation = t.translation.lerp(l.target.translation, s);
        t.rotation = t.rotation.lerp(l.target.rotation, s);
        // t.scale = t.scale.lerp(l.target.scale, s); Removed since current scale is not sent
    }
}

#[derive(Component)]
pub struct LocalTurretModelLoadHandle(pub Handle<Gltf>, pub Entity);

pub fn spawn_local_turret(
    commands: &mut Commands,
    ass: &Res<AssetServer>,
    transform: &Transform,
    owner_entity: Entity,
) -> Entity {
    let turret_gltf = ass.load("../../shared/assets/turrets/turret_large/turret_large.gltf");
    let turret_entity = commands
        .spawn((
            SpatialBundle { ..default() },
            LocalTurretModelLoadHandle(turret_gltf, owner_entity),
        ))
        .insert(transform.clone())
        .id();
    turret_entity
}

pub fn handle_ship_model_load(
    mut commands: Commands,
    query: Query<(Entity, &ShipModelLoadHandle)>,
    assets_gltf: Res<Assets<Gltf>>,
    assets_gltfnode: Res<Assets<GltfNode>>,
    ass: Res<AssetServer>,
    mut effects: ResMut<Assets<EffectAsset>>,
) {
    for (entity, handle) in query.iter() {
        if let Some(gltf) = assets_gltf.get(&handle.0) {
            let mut gradient = Gradient::new();
            gradient.add_key(0.0, Vec4::new(0.0, 1.0, 1.0, 1.0) * 3.0);
            gradient.add_key(1.0, Vec4::new(0.0, 1.0, 1.0, 0.0));

            println!("TEST");
            let spawner = Spawner::rate(100.0.into());
            let effect = effects.add(
                EffectAsset {
                    name: "Impact".into(),
                    capacity: 32768,
                    spawner,
                    ..Default::default()
                }
                .init(InitLifetimeModifier { lifetime: 1.0 })
                .init(InitVelocitySphereModifier {
                    speed: 0.0.into(),
                    ..Default::default()
                })
                .init(InitPositionSphereModifier {
                    radius: 0.75,
                    dimension: ShapeDimension::Volume,
                    ..Default::default()
                })
                .render(SizeOverLifetimeModifier {
                    gradient: Gradient::constant(Vec2::splat(0.05)),
                })
                .render(ColorOverLifetimeModifier { gradient })
                .render(BillboardModifier {}),
            );

            println!("Loaded GLTF, spawning...");
            // spawn the first scene in the file
            let model = commands
                .spawn(SceneBundle {
                    scene: gltf.scenes[0].clone(),
                    ..Default::default()
                })
                .id();
            let mut thruster_points: Vec<Entity> = vec![];
            let mut turrets: Vec<Entity> = vec![];

            for node_name in gltf.named_nodes.keys().into_iter() {
                if node_name.contains("forward_thrusters") {
                    if let Some(node) = assets_gltfnode.get(&gltf.named_nodes[node_name]) {
                        let thruster = commands
                            .spawn(ParticleEffectBundle::new(effect.clone()))
                            .insert(node.transform)
                            .id();
                        thruster_points.push(thruster);
                    }
                }
                if node_name.contains("turret_pad_large") {
                    if let Some(node) = assets_gltfnode.get(&gltf.named_nodes[node_name]) {
                        let turret =
                            spawn_local_turret(&mut commands, &ass, &node.transform, entity);
                        turrets.push(turret);
                    }
                }
            }

            commands
                .entity(entity)
                .push_children(&[model])
                .push_children(&thruster_points)
                .push_children(&turrets)
                .remove::<ShipModelLoadHandle>();
        }
    }
}

pub fn handle_turret_model_load(
    mut commands: Commands,
    query: Query<(Entity, &LocalTurretModelLoadHandle)>,
    assets_gltf: Res<Assets<Gltf>>,
) {
    for (entity, handle) in query.iter() {
        if let Some(gltf) = assets_gltf.get(&handle.0) {
            // spawn the first scene in the file
            let owner_entity_handle = Box::new(handle.1);

            commands
                .entity(*owner_entity_handle)
                .insert(Name::new("TurretOwnerEntity"));
            let model = commands
                .spawn(HookedSceneBundle {
                    scene: SceneBundle {
                        scene: gltf.scenes[0].clone(),
                        ..default()
                    },
                    hook: SceneHook::new(move |entity, cmds| {
                        match entity.get::<Name>().map(|t| t.as_str()) {
                            Some("base") => {
                                cmds.insert((
                                    Turret { ..default() },
                                    TurretOwner::new(*owner_entity_handle),
                                ));
                                println!("Inserting Turret");
                                cmds
                            }
                            Some("pivot") => {
                                cmds.insert((Barrel {},));
                                println!("Inserting Barrel");
                                cmds
                            }
                            // Some("barrel_end") => cmds.insert(),
                            _ => cmds,
                        };
                    }),
                })
                .id();

            commands
                .entity(entity)
                .push_children(&[model])
                .remove::<LocalTurretModelLoadHandle>();
        }
    }
}
