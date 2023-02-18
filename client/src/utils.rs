use bevy::{
    gltf::{Gltf, GltfNode},
    prelude::{
        Assets, BuildChildren, Commands, Component, Entity, Query, Res, ResMut, Transform, Vec2,
        Vec4,
    },
    scene::SceneBundle,
    time::Time,
};
use bevy_hanabi::{
    BillboardModifier, ColorOverLifetimeModifier, EffectAsset, Gradient, ParticleEffectBundle,
    ParticleLifetimeModifier, PositionSphereModifier, ShapeDimension, SizeOverLifetimeModifier,
    Spawner,
};

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

pub fn spawn_gltf_objects(
    mut commands: Commands,
    query: Query<(Entity, &ShipModelLoadHandle)>,
    assets_gltf: Res<Assets<Gltf>>,
    assets_gltfnode: Res<Assets<GltfNode>>,
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
                .init(ParticleLifetimeModifier { lifetime: 1.0 })
                .init(PositionSphereModifier {
                    radius: 0.75,
                    speed: 0.0.into(),
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
            }

            commands
                .entity(entity)
                .push_children(&[model])
                .push_children(&thruster_points)
                .remove::<ShipModelLoadHandle>();
        }
    }
}
