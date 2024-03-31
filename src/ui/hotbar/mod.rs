use bevy::asset::{AssetServer, Handle};
use bevy::hierarchy::{BuildChildren, ChildBuilder};
use bevy::prelude::{AlignItems, BackgroundColor, Bundle, ButtonBundle, Color, Commands, default, Display, Image, ImageBundle, JustifyContent, NodeBundle, Res, Style, Text, TextBundle, TextStyle, UiImage, UiRect, Val};
use bevy::ui::PositionType;

use crate::common::{PRIMARY_COLOR, PRIMARY_COLOR_FADED};
use crate::ui::components::OriginalColor;
use crate::ui::hotbar::components::{CloseIconMarker, ImportIconMarker, OpenIconMarker, SaveIconMarker, TopHotbarMarker};

pub(crate) mod components;

fn spawn_button_icon<T: Bundle>(container: &mut ChildBuilder, icon: Handle<Image>, color: Color, marker: T) {
    container.spawn((
        ButtonBundle {
            style: Style {
                width: Val::Px(32.),
                height: Val::Px(32.),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            background_color: BackgroundColor::from(color),
            ..default()
        },
        OriginalColor(color),
        marker
    )).with_children(|icon_container| {
        icon_container.spawn(
            ImageBundle {
                style: Style {
                    ..default()
                },
                image: UiImage::from(icon),
                ..default()
            }
        );
    });
}

pub fn spawn_hotbar(mut commands: Commands, asset_server: Res<AssetServer>) {
    build_hotbar(&mut commands, &asset_server);
}

pub fn build_hotbar(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    let grass = asset_server.load("grass.png");
    let font = asset_server.load("fonts/unifont.ttf");

    commands.spawn(
        NodeBundle {
            style: Style {
                display: Display::Flex,
                justify_content: JustifyContent::SpaceBetween,
                width: Val::Percent(100.),
                padding: UiRect::px(3., 0., 3., 3.),
                height: Val::Px(32.),
                ..default()
            },
            background_color: BackgroundColor::from(PRIMARY_COLOR),
            ..default()
        },
    ).with_children(|parent| {
        parent.spawn((
            NodeBundle {
                style: Style {
                    height: Val::Percent(100.),
                    column_gap: Val::Px(5.),
                    display: Display::Flex,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            },
            TopHotbarMarker {}
        ))
            .with_children(|top_left| {
                top_left.spawn(
                    ImageBundle {
                        style: Style {
                            width: Val::Px(24.),
                            height: Val::Px(24.),
                            ..default()
                        },
                        image: UiImage::new(grass.clone()),
                        ..default()
                    }
                );

                top_left.spawn(
                    TextBundle {
                        style: Style {
                            height: Val::Percent(100.),
                            top: Val::Px(6.),
                            ..default()
                        },
                        text: Text::from_section(
                            "CDDA Map Editor",
                            TextStyle {
                                font: font.clone(),
                                font_size: 12.,
                                color: Color::hex("#FFFFFF").unwrap(),
                            }),
                        ..default()
                    },
                );
            });
    }).with_children(|parent| {
        parent.spawn(NodeBundle {
            style: Style {
                height: Val::Percent(100.),
                column_gap: Val::Px(5.),
                display: Display::Flex,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        }).with_children(|top_right| {
            spawn_button_icon(top_right, asset_server.load("icons/close.png"), PRIMARY_COLOR, CloseIconMarker {});
        });
    });

    commands.spawn(
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.),
                height: Val::Px(32.),
                top: Val::Px(30.),
                ..default()
            },
            background_color: BackgroundColor::from(PRIMARY_COLOR_FADED),
            ..default()
        }
    ).with_children(|parent| {
        parent.spawn(NodeBundle {
            style: Style {
                width: Val::Px(200.),
                height: Val::Percent(100.),
                display: Display::Flex,
                ..default()
            },
            ..default()
        }).with_children(|icons_container| {
            spawn_button_icon(icons_container, asset_server.load("icons/floppy-disk.png"), PRIMARY_COLOR_FADED, SaveIconMarker {});
            //spawn_button_icon(icons_container, asset_server.load("icons/upload-file.png"), PRIMARY_COLOR_FADED);
            spawn_button_icon(icons_container, asset_server.load("icons/download-file.png"), PRIMARY_COLOR_FADED, ImportIconMarker {});
            spawn_button_icon(icons_container, asset_server.load("icons/new-folder.png"), PRIMARY_COLOR_FADED, OpenIconMarker {});
            //spawn_button_icon(icons_container, asset_server.load("icons/recycle-bin.png"), ERROR)
        });
    });
}