mod capture_point;
mod client;
mod networking;
mod player;
mod server;
mod ship;
mod team;
use bevy_egui::{egui::Window, EguiContext, EguiPlugin};
use bevy_inspector_egui::{Inspectable, RegisterInspectable, WorldInspectorPlugin};
use client::ClientPlugin;

use bevy::{
    pbr::NotShadowCaster,
    prelude::{
        default, shape, App, AssetPlugin, Assets, Camera3dBundle, Color, Commands,
        DirectionalLight, DirectionalLightBundle, MaterialMeshBundle, Mesh, OrthographicProjection,
        PbrBundle, PluginGroup, Quat, Query, ResMut, StandardMaterial, Transform, Vec3,
    },
    utils::HashSet,
    DefaultPlugins,
};
use capture_point::CapturePointPlugin;
use naia_bevy_client::Client;
use naia_bevy_server::{Server, ServerAddrs};
use networking::{
    channels::Channels,
    protocol::{Auth, NetworkPosition, Protocol},
};
use player::PlayerPlugin;
use server::ServerPlugin;
use ship::{space_ship::SpaceShip, SpaceShipPlugin};
use std::{collections::HashMap, f32::consts::PI};

use crate::{
    capture_point::capture_point::{CaptureSphere, ForceFieldMaterial},
    player::player::Player,
    team::team_enum::Team,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            // Tell the asset server to watch for asset changes on disk:
            watch_for_changes: true,
            ..default()
        }))
        .add_plugin(EguiPlugin)
        .add_system(ui_example)
        .add_system(trala)
        // .add_plugin(SpaceShipPlugin)
        // .add_plugin(CapturePointPlugin)
        // .add_plugin(PlayerPlugin)
        .add_startup_system(setup)
        // Client, Server
        .add_plugin(ServerPlugin)
        .add_plugin(ClientPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .register_inspectable::<NetworkPosition>()
        .run();
}

fn ui_example(
    mut egui_context: ResMut<EguiContext>,
    mut commands: Commands,
    mut server: Server<Protocol, Channels>,
    mut client: Client<Protocol, Channels>,
) {
    Window::new("Hello").show(egui_context.ctx_mut(), |ui| {
        ui.label("world");
        if ui.button("Start Server").clicked() {
            println!("Starting Server");
            // Naia Server initialization
            let server_addresses = ServerAddrs::new(
                "127.0.0.1:14191"
                    .parse()
                    .expect("could not parse Signaling address/port"),
                // IP Address to listen on for UDP WebRTC data channels
                "127.0.0.1:14192"
                    .parse()
                    .expect("could not parse WebRTC data address/port"),
                // The public WebRTC IP address to advertise
                "http://127.0.0.1:14192",
            );

            server.listen(&server_addresses);

            // Create a new, singular room, which will contain Users and Entities that they
            // can receive updates from
            let main_room_key = server.make_room().key();

            // Resources
            commands.insert_resource(server::resources::Global {
                main_room_key,
                user_to_prediction_map: HashMap::new(),
                player_last_command: HashMap::new(),
            })
        }

        if ui.button("Join Server").clicked() {
            client.auth(Auth::new("charlie", "12345"));
            client.connect("http://127.0.0.1:14191");
        }
    });
}

fn trala(mut egui_context: ResMut<EguiContext>, query: Query<&NetworkPosition>) {
    Window::new("Hellos").show(egui_context.ctx_mut(), |ui| {
        for p in query.iter() {
            ui.label(p.x.to_string());
            ui.label(p.y.to_string());
            ui.label(p.z.to_string());
        }
    });
}

fn setup(
    mut commands: Commands,
    // mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<StandardMaterial>>,
    // mut force_field_materials: ResMut<Assets<ForceFieldMaterial>>,
) {
    // commands
    //     .spawn(PbrBundle {
    //         mesh: meshes.add(shape::Cube { size: 1. }.into()),
    //         material: materials.add(Color::GREEN.into()),
    //         ..default()
    //     })
    //     .insert(SpaceShip { hp: 20 })
    //     .insert(Player { team: Team::Blue });

    // commands
    //     .spawn(MaterialMeshBundle {
    //         mesh: meshes.add(
    //             shape::Icosphere {
    //                 radius: 3.,
    //                 subdivisions: 8,
    //             }
    //             .into(),
    //         ),
    //         material: force_field_materials.add(ForceFieldMaterial {}),
    //         ..default()
    //     })
    //     .insert(NotShadowCaster)
    //     .insert(CaptureSphere {
    //         radius: 3.,
    //         progress: 0.0,
    //         attackers: HashSet::new(),
    //         owner: Team::Neutral,
    //     });

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 6., 12.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
        ..default()
    });

    const HALF_SIZE: f32 = 10.0;
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            // Configure the projection to better fit the scene
            shadow_projection: OrthographicProjection {
                left: -HALF_SIZE,
                right: HALF_SIZE,
                bottom: -HALF_SIZE,
                top: HALF_SIZE,
                near: -10.0 * HALF_SIZE,
                far: 10.0 * HALF_SIZE,
                ..default()
            },
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
}
