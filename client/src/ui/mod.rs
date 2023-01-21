use bevy::{
    prelude::{
        default, App, AssetServer, BuildChildren, Commands, ImageBundle, NodeBundle, Plugin, Res,
    },
    ui::{AlignItems, JustifyContent, Size, Style, Val},
};

pub struct GameUIPlugin;

impl Plugin for GameUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup);
    }
}

const CROSSHAIR_SIZE: f32 = 20.0;

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
