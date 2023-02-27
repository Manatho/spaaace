use bevy::{
    gltf::Gltf,
    prelude::{
        default, AssetServer, Assets, BuildChildren, Commands, Component, Entity, Handle, Name,
        Query, Res, SpatialBundle, Transform,
    },
    scene::SceneBundle,
};
use bevy_scene_hook::{HookedSceneBundle, SceneHook};

use crate::util::spring::LocalSpring;

use super::{PartOfTurret, Turret, TurretBarrel, TurretBase, TurretOwner, TurretPivot};

#[derive(Component)]
pub struct TurretModelLoadHandle(pub Handle<Gltf>, pub Entity);

pub fn spawn_turret(
    commands: &mut Commands,
    ass: &Res<AssetServer>,
    transform: &Transform,
    owner_entity: Entity,
) -> Entity {
    let turret_gltf = ass.load("../../shared/assets/turrets/turret_large/turret_large.gltf");
    let turret_entity = commands
        .spawn((
            SpatialBundle { ..default() },
            TurretModelLoadHandle(turret_gltf, owner_entity),
        ))
        .insert(transform.clone())
        .id();
    turret_entity
}

pub fn handle_turret_model_load(
    mut commands: Commands,
    query: Query<(Entity, &TurretModelLoadHandle)>,
    assets_gltf: Res<Assets<Gltf>>,
) {
    for (entity, handle) in query.iter() {
        if let Some(gltf) = assets_gltf.get(&handle.0) {
            // spawn the first scene in the file
            let owner_entity_handle = Box::new(handle.1);

            commands
                .entity(*owner_entity_handle)
                .insert(Name::new("TurretOwnerEntity"));

            let turret_entity = commands
                .entity(entity)
                .insert((
                    Name::new("Turret"),
                    Turret { ..default() },
                    TurretOwner::new(*owner_entity_handle),
                ))
                .id();

            let model = commands
                .spawn(HookedSceneBundle {
                    scene: SceneBundle {
                        scene: gltf.scenes[0].clone(),
                        ..default()
                    },
                    hook: SceneHook::new(move |entity, cmds| {
                        match entity.get::<Name>().map(|t| t.as_str()) {
                            Some("base") => {
                                cmds.insert((TurretBase {}, PartOfTurret::new(turret_entity)));
                                cmds
                            }
                            Some("pivot") => {
                                cmds.insert((TurretPivot {}, PartOfTurret::new(turret_entity)));
                                cmds
                            }
                            Some(x) => {
                                if x.contains("barrel") {
                                    cmds.insert((
                                        TurretBarrel {},
                                        PartOfTurret::new(turret_entity),
                                        LocalSpring {
                                            resting_transform: entity
                                                .get::<Transform>()
                                                .unwrap_or_else(|| &Transform::IDENTITY)
                                                .clone(),
                                        },
                                    ));
                                }
                                cmds
                            }
                            _ => cmds,
                        };
                    }),
                })
                .id();

            commands
                .entity(entity)
                .push_children(&[model])
                .remove::<TurretModelLoadHandle>();
        }
    }
}
