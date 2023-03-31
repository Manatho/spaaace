mod capture_point;

use bevy::{
    pbr::NotShadowCaster,
    prelude::{
        shape::Icosphere, App, AssetServer, Assets, Color, Commands, EventReader, Handle,
        MaterialMeshBundle, MaterialPlugin, Mesh, Plugin, Query, Res, ResMut, Transform,
    },
    time::Time,
    utils::default,
};

use spaaaace_shared::{team::team_enum::Team, Lobby, ServerMessages};

use self::capture_point::ForceFieldMaterial;

pub struct ClientCapturePointPlugin;
impl Plugin for ClientCapturePointPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(on_capture_point_spawned)
            .add_system(on_capture_point_updated)
            .add_plugin(MaterialPlugin::<ForceFieldMaterial>::default());
    }
}

fn on_capture_point_spawned(
    mut commands: Commands,
    mut lobby: ResMut<Lobby>,
    mut event_reader: EventReader<ServerMessages>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut force_field_materials: ResMut<Assets<ForceFieldMaterial>>,
    time: Res<Time>,
    ass: Res<AssetServer>,
) {
    for event in event_reader.iter() {
        match event {
            ServerMessages::CapturePointSpawned {
                position,
                rotation,
                id,
                owner,
                progress: _,
            } => {
                let capture_entity = commands
                    .spawn(MaterialMeshBundle {
                        mesh: meshes.add(
                            Icosphere {
                                radius: 50.,
                                subdivisions: 8,
                                ..Default::default()
                            }
                            .try_into()
                            .unwrap(),
                        ),
                        material: force_field_materials.add(ForceFieldMaterial {
                            color: match owner {
                                Team::Neutral => Color::WHITE,
                                Team::Red => Color::RED,
                                Team::Blue => Color::BLUE,
                            },
                            prev_color: Color::WHITE,
                            last_color_change: time.elapsed_seconds(),
                            color_texture: Some(ass.load("hex_grid.jpg")),
                        }),
                        transform: Transform {
                            rotation: *rotation,
                            translation: *position,
                            ..Default::default()
                        },
                        ..default()
                    })
                    .insert(NotShadowCaster)
                    .id();

                lobby.capture_points.insert(*id, capture_entity);
            }
            _ => {}
        }
    }
}

fn on_capture_point_updated(
    mut event_reader: EventReader<ServerMessages>,
    mut force_field_materials: ResMut<Assets<ForceFieldMaterial>>,
    lobby: ResMut<Lobby>,
    time: Res<Time>,
    query: Query<&Handle<ForceFieldMaterial>>,
) {
    for event in event_reader.iter() {
        match event {
            ServerMessages::CapturePointUpdate {
                id,
                owner,
                attacker: _,
                progress: _,
            } => {
                if let Some(entity) = lobby.capture_points.get(&id) {
                    match query.get(*entity) {
                        Ok(material) => {
                            if let Some(material) = force_field_materials.get_mut(material) {
                                let next_color = match owner {
                                    Team::Neutral => Color::WHITE,
                                    Team::Red => Color::RED,
                                    Team::Blue => Color::BLUE,
                                };
                                if material.color != next_color {
                                    material.prev_color = material.color;
                                    material.color = next_color;
                                    material.last_color_change = time.elapsed_seconds();

                                    println!(
                                        "{:?} {:?} {:?}",
                                        material.color,
                                        material.prev_color,
                                        material.last_color_change
                                    );
                                }
                            }
                        }
                        _ => (),
                    }
                }
            }
            _ => {}
        }
    }
}
