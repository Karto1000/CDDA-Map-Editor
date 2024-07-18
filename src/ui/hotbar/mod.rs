use bevy::asset::{AssetServer, Handle};
use bevy::hierarchy::{BuildChildren, ChildBuilder};
use bevy::prelude::{AlignItems, BackgroundColor, Bundle, ButtonBundle, Color, Commands, default, Display, FlexDirection, Image, ImageBundle, JustifyContent, NodeBundle, Res, Style, Text, TextBundle, TextStyle, UiImage, UiRect, Val};
use bevy::ui::PositionType;

use crate::program::data::Program;
use crate::ui::{HoverEffect, OriginalColor};
use crate::ui::hotbar::components::{CloseIconMarker, CustomTitleBarMarker, ImportIconMarker, OpenIconMarker, ProjectSettingsMarker, SaveIconMarker, SettingsIconMarker, TileSettingsMarker, TopHotbarMarker};

pub(crate) mod components;

fn spawn_button_icon<T: Bundle>(
    container: &mut ChildBuilder,
    program: &Res<Program>,
    icon: Handle<Image>,
    color: Color,
    marker: T,
) {
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
        HoverEffect {
            original_color: color,
            hover_color: program.config.style.selected,
        },
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

pub fn spawn_hotbar(
    mut commands: Commands,
    r_asset_server: Res<AssetServer>,
    r_editor_data: Res<Program>,
) {
    build_hotbar(&mut commands, &r_asset_server, &r_editor_data);
}

pub fn build_hotbar(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    editor_data: &Res<Program>,
) {
    let grass = asset_server.load("grass.png");
    let font = asset_server.load("fonts/unifont.ttf");

    commands.spawn((
                       ButtonBundle {
                           style: Style {
                               display: Display::Flex,
                               justify_content: JustifyContent::SpaceBetween,
                               width: Val::Percent(100.),
                               padding: UiRect::px(3., 0., 3., 3.),
                               height: Val::Px(32.),
                               ..default()
                           },
                           background_color: BackgroundColor::from(editor_data.config.style.gray_dark),
                           ..default()
                       }, CustomTitleBarMarker {}),
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
            spawn_button_icon(
                top_right,
                editor_data,
                asset_server.load("icons/close.png"),
                editor_data.config.style.gray_dark,
                CloseIconMarker {},
            );
        });
    });

    commands.spawn(
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                width: Val::Percent(100.),
                height: Val::Px(32.),
                top: Val::Px(30.),
                ..default()
            },
            background_color: BackgroundColor::from(editor_data.config.style.gray_darker),
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
            spawn_button_icon(
                icons_container,
                editor_data,
                asset_server.load("icons/floppy-disk.png"),
                editor_data.config.style.gray_darker,
                SaveIconMarker {},
            );
            //spawn_button_icon(icons_container, asset_server.load("icons/upload-file.png"), PRIMARY_COLOR_FADED);
            spawn_button_icon(
                icons_container,
                editor_data,
                asset_server.load("icons/download-file.png"),
                editor_data.config.style.gray_darker,
                ImportIconMarker {},
            );
            spawn_button_icon(
                icons_container,
                editor_data,
                asset_server.load("icons/new-folder.png"),
                editor_data.config.style.gray_darker,
                OpenIconMarker {},
            );
            //spawn_button_icon(icons_container, asset_server.load("icons/recycle-bin.png"), ERROR)
        });

        parent.spawn(NodeBundle {
            style: Style {
                width: Val::Auto,
                height: Val::Percent(100.),
                display: Display::Flex,
                ..default()
            },
            ..default()
        }).with_children(|icons_container| {
            spawn_button_icon(
                icons_container,
                editor_data,
                asset_server.load("icons/variation.png"),
                editor_data.config.style.gray_darker,
                TileSettingsMarker,
            );

            spawn_button_icon(
                icons_container,
                editor_data,
                asset_server.load("icons/edit.png"),
                editor_data.config.style.gray_darker,
                ProjectSettingsMarker,
            );

            spawn_button_icon(
                icons_container,
                editor_data,
                asset_server.load("icons/cog.png"),
                editor_data.config.style.gray_darker,
                SettingsIconMarker,
            );
        });
    });
}