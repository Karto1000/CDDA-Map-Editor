use bevy::asset::AssetServer;
use bevy::hierarchy::BuildChildren;
use bevy::prelude::{Commands, Component, default, NodeBundle, Res, Text, TextBundle, TextStyle};
use bevy::ui::{BackgroundColor, PositionType, Style, UiRect, Val};

use crate::common::{PRIMARY_COLOR, PRIMARY_COLOR_FADED};

#[derive(Component)]
pub struct ScrollingList {
    scroll_progress: u32,
}

pub fn spawn_tile_selector(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    build_tile_selector(&mut commands, &asset_server);
}

pub fn build_tile_selector(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    let font = asset_server.load("fonts/unifont.ttf");

    commands.spawn(
        NodeBundle {
            style: Style {
                width: Val::Px(400.),
                height: Val::Px(330.),
                position_type: PositionType::Absolute,
                bottom: Val::Px(5.),
                left: Val::Px(5.),
                ..default()
            },
            background_color: BackgroundColor::from(PRIMARY_COLOR_FADED),
            ..default()
        }
    ).with_children(|parent| {
        parent.spawn(
            NodeBundle {
                style: Style {
                    padding: UiRect::all(Val::Px(8.)),
                    height: Val::Px(42.),
                    width: Val::Percent(100.),
                    ..default()
                },
                background_color: BackgroundColor::from(PRIMARY_COLOR),
                ..default()
            }).with_children(|parent| {
            parent.spawn(
                TextBundle {
                    text: Text::from_section(
                        "Tiles",
                        TextStyle {
                            font,
                            font_size: 24.,
                            ..default()
                        },
                    ),
                    ..default()
                }
            );
        });
    });
}