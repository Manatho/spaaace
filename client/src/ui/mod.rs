use bevy::{
    prelude::{
        default, App, AssetServer, BuildChildren, Children, Color, Commands, Component,
        DespawnRecursiveExt, Entity, ImageBundle, Input, KeyCode, NodeBundle, Plugin, Query, Res,
        ResMut, TextBundle, Visibility, With,
    },
    text::TextStyle,
    ui::{AlignItems, FlexDirection, JustifyContent, Node, PositionType, Size, Style, Val},
    window::{CursorGrabMode, Windows},
};

use spaaaace_shared::Lobby;

use crate::game_state::ClientGameState;

pub struct GameUIPlugin;

impl Plugin for GameUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
            .add_system(input)
            .add_system(update_pause_mode)
            .add_system(scoreboard);
    }
}

const CROSSHAIR_SIZE: f32 = 20.0;

#[derive(Component)]
struct PauseBackdrop;

#[derive(Component)]
struct Scoreboard;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // root node
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                    background_color: Color::Hsla {
                        hue: 0.0,
                        saturation: 0.0,
                        lightness: 0.0,
                        alpha: 0.5,
                    }
                    .into(),
                    ..Default::default()
                })
                .insert(PauseBackdrop {})
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Paused.",
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 100.0,
                            color: Color::WHITE,
                        },
                    ));
                });

            parent
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(30.0), Val::Percent(50.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        position_type: PositionType::Absolute,
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    background_color: Color::RED.into(),
                    ..Default::default()
                })
                .insert(Scoreboard {});

            parent.spawn(ImageBundle {
                style: Style {
                    size: Size::new(Val::Px(CROSSHAIR_SIZE), Val::Px(CROSSHAIR_SIZE)),
                    ..default()
                },
                image: asset_server.load("crosshair.png").into(),
                ..default()
            });
        });
}

fn input(keys: Res<Input<KeyCode>>, mut game_state: ResMut<ClientGameState>) {
    if keys.just_pressed(KeyCode::Escape) {
        game_state.is_paused = !game_state.is_paused;
    }
}

fn update_pause_mode(
    game_state: Res<ClientGameState>,
    mut query: Query<(&PauseBackdrop, &mut Visibility)>,
    mut windows: ResMut<Windows>,
) {
    let window = windows.get_primary_mut().unwrap();

    if game_state.is_changed() {
        for (_, mut vis) in query.iter_mut() {
            vis.is_visible = game_state.is_paused;
        }

        let grab_mode = match game_state.is_paused {
            true => CursorGrabMode::None,
            false => CursorGrabMode::Locked,
        };

        window.set_cursor_grab_mode(grab_mode);
        window.set_cursor_visibility(game_state.is_paused);
    }
}

fn scoreboard(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    lobby: Res<Lobby>,
    mut query: Query<(Entity, &Node, Option<&Children>, &mut Visibility), With<Scoreboard>>,
    asset_server: Res<AssetServer>,
) {
    let lobby_players = lobby.players.clone();
    for (node_ent, _node, children, mut vis) in query.iter_mut() {
        vis.is_visible = keys.pressed(KeyCode::Tab);

        if lobby.is_changed() == false {
            continue;
        }

        if children.is_some() {
            for child in children.unwrap() {
                commands.entity(*child).despawn_recursive();
            }
        }
        for ply in lobby_players.keys() {
            let entry = format!("{}", ply);
            let item = commands
                .spawn(TextBundle::from_section(
                    entry,
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 20.0,
                        color: Color::WHITE,
                    },
                ))
                .id();
            commands.entity(node_ent).add_child(item);
        }
    }
}
