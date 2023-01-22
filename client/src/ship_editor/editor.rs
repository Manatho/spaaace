use std::{
    f32::consts::PI,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use bevy::{
    app::App,
    core_pipeline::{bloom::BloomSettings, fxaa::Fxaa},
    diagnostic::FrameTimeDiagnosticsPlugin,
    ecs::{
        query,
        system::{CommandQueue, SystemState},
    },
    prelude::{
        default, info, AmbientLight, AppTypeRegistry, AssetServer, Camera, Camera3dBundle, Color,
        Commands, DirectionalLight, DirectionalLightBundle, Entity, EventReader, EventWriter, Name,
        PluginGroup, Quat, Query, Res, ResMut, Resource, Transform, Vec3, World,
    },
    scene::{DynamicScene, DynamicSceneBundle},
    tasks::IoTaskPool,
    window::{CursorGrabMode, WindowDescriptor, WindowPlugin},
    DefaultPlugins,
};

use bevy_egui::{egui, EguiContext, EguiPlugin};

use rand::Rng;
use rfd::FileDialog;

use bevy_inspector_egui::prelude::*;

pub fn run() {
    App::default()
        // Plugins
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "Ship Editor".to_string(),
                width: 640. * 2.0,
                height: 320. * 2.0,
                ..default()
            },
            ..default()
        }))
        .add_plugin(EguiPlugin)
        .add_plugins(bevy_mod_picking::DefaultPickingPlugins)
        .add_system(menu_bar)
        // .add_system(hierarchy)
        .insert_resource(ShipSaveLoadState { current_file: None })
        .add_event::<SaveEvent>()
        .add_system(save_scene_system)
        .run();
}

fn init(mut commands: Commands, mut ambient_light: ResMut<AmbientLight>, ass: Res<AssetServer>) {
    commands
        .spawn(Camera3dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },

            transform: Transform::from_xyz(0.0, 6., -12.0)
                .looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
            ..default()
        })
        .insert(BloomSettings { ..default() })
        .insert(Fxaa { ..default() })
        .insert(bevy_mod_picking::PickingCameraBundle::default());

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
        ..default()
    });

    ambient_light.color = Color::hsl(180.0, 1.0, 1.0);
    ambient_light.brightness = 0.2;
}

#[derive(Resource)]
struct ShipSaveLoadState {
    current_file: Option<PathBuf>,
}

struct SaveEvent;

fn menu_bar(
    world: &mut World,
    // mut egui_context: ResMut<EguiContext>,
    // mut state: ResMut<ShipSaveLoadState>,
    // asset_server: Res<AssetServer>,
    // mut commands: Commands,
    // query: Query<(Entity, &Transform)>,
    // query_names: Query<&Name>,
    // mut save_events: EventWriter<SaveEvent>,
) {
    let egui_context = world
        .resource_mut::<bevy_egui::EguiContext>()
        .ctx_mut()
        .clone();
    let mut world_state = SystemState::<(
        Commands,
        ResMut<ShipSaveLoadState>,
        ResMut<AssetServer>,
        EventWriter<SaveEvent>,
    )>::new(world);
    let (
        //
        mut commands,
        mut state,
        mut asset_server,
        mut save_events,
    ) = world_state.get_mut(world);

    egui::TopBottomPanel::top("menu_bar").show(&egui_context, |ui| {
        egui::menu::bar(ui, |ui| {
            if ui.button("Load Ship").clicked() {
                println!("test");

                let file = FileDialog::new()
                    .add_filter("Spaaaceship (.ship)", &["ron"])
                    .set_directory("/")
                    .pick_file();
                state.current_file = file.clone();

                commands
                    .spawn(DynamicSceneBundle {
                        scene: asset_server.load(file.unwrap()),
                        ..Default::default()
                    })
                    .insert(bevy_mod_picking::PickableBundle::default());
            }

            let save_btn =
                ui.add_enabled(state.current_file.is_some(), egui::Button::new("Save ship"));
            // save_btn.enabled(false);
            if save_btn.clicked() {
                save_events.send(SaveEvent {});
            }
        })
    });

    egui::SidePanel::left("hierarchy").show(&egui_context, |ui| {
        bevy_inspector_egui::bevy_inspector::ui_for_world_entities(world, ui);
    });

    // egui::SidePanel::right("inspector").show(&egui_context, |ui| {
    //     bevy_inspector_egui::bevy_inspector::ui_for_world_entities(world, ui);
    // });
}

fn save_scene_system(
    mut save_events: EventReader<SaveEvent>,
    world: &World,
    state: Res<ShipSaveLoadState>,
) {
    for event in save_events.iter() {
        // // Scenes can be created from any ECS World. You can either create a new one for the scene or
        // // use the current World.
        // let mut scene_world = World::new();
        // let mut component_b = ComponentB::from_world(world);
        // component_b.value = "hello".to_string();
        // scene_world.spawn((
        //     component_b,
        //     ComponentA { x: 1.0, y: 2.0 },
        //     Transform::IDENTITY,
        // ));
        // scene_world.spawn(ComponentA { x: 3.0, y: 4.0 });

        // The TypeRegistry resource contains information about all registered types (including
        // components). This is used to construct scenes.
        let type_registry = world.resource::<AppTypeRegistry>();
        let scene = DynamicScene::from_world(&world, type_registry);

        // Scenes can be serialized like this:
        let serialized_scene = scene.serialize_ron(type_registry).unwrap();

        // Showing the scene in the console
        info!("{}", serialized_scene);

        // // Writing the scene to a new file. Using a task to avoid calling the filesystem APIs in a system
        // // as they are blocking
        // // This can't work in WASM as there is no filesystem access
        let save_path = state.current_file.clone().unwrap();
        #[cfg(not(target_arch = "wasm32"))]
        IoTaskPool::get()
            .spawn(async move {
                // Write the scene RON data to file
                File::create(save_path)
                    .and_then(|mut file| file.write(serialized_scene.as_bytes()))
                    .expect("Error while writing scene to file");
            })
            .detach();
    }
}
