use bevy::asset::{AssetServer, Handle};
use bevy::math::Vec2;
use bevy::prelude::{AlignItems, BackgroundColor, BuildChildren, Bundle, Button, ButtonBundle, Changed, ChildBuilder, Color, Commands, Component, default, GlobalTransform, Image, ImageBundle, NodeBundle, Query, Res, ResMut, TextBundle, Vec3Swizzles, Visibility, Window, With};
use bevy::text::{Text, TextStyle};
use bevy::ui::{Display, FlexDirection, Interaction, JustifyContent, Node, Style, UiImage, UiRect, Val};
use bevy::window::PrimaryWindow;

use crate::IsCursorCaptured;

pub const PRIMARY_COLOR: Color = Color::rgb(0.19, 0.21, 0.23);
pub const PRIMARY_COLOR_FADED: Color = Color::rgb(0.23, 0.25, 0.27);
pub const PRIMARY_COLOR_SELECTED: Color = Color::rgb(0.63, 0.70, 0.76);
pub const ERROR: Color = Color::rgba(0.79, 0.2, 0.21, 0.5);

#[derive(Component)]
pub struct OriginalColor(pub Color);

#[derive(Component)]
pub struct CloseIconMarker;

#[derive(Component)]
pub struct SaveIconMarker;

#[derive(Component)]
pub struct ImportIconMarker;

#[derive(Component)]
pub struct TopHotbarMarker;

#[derive(Component)]
pub struct TabContainerMarker;

#[derive(Component)]
pub struct AddTabButtonMarker;

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
        parent.spawn((NodeBundle {
            style: Style {
                height: Val::Percent(100.),
                column_gap: Val::Px(5.),
                display: Display::Flex,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        }, TopHotbarMarker {}))
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
    })
        .with_children(|parent| {
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
            //spawn_button_icon(icons_container, asset_server.load("icons/new-folder.png"), PRIMARY_COLOR_FADED);
            //spawn_button_icon(icons_container, asset_server.load("icons/recycle-bin.png"), ERROR)
        });
    });
}

pub fn button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &OriginalColor
        ),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color, original_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = BackgroundColor::from(original_color.0).into();
            }
            Interaction::Hovered => {
                *color = BackgroundColor::from(Color::rgba(
                    color.0.r() + 0.2,
                    color.0.g() + 0.2,
                    color.0.b() + 0.2, color.0.a(),
                )).into();
            }
            Interaction::None => {
                *color = BackgroundColor::from(original_color.0).into();
            }
        }
    }
}

pub fn check_ui_interaction(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut res_cursor: ResMut<IsCursorCaptured>,
    node_query: Query<(&Node, &GlobalTransform, &Visibility)>,
) {
    let cursor_position = windows.single().cursor_position().unwrap_or(Vec2::default());
    res_cursor.0 = node_query.iter()
        .any(|(&node, &transform, &visibility)| {
            let node_position = transform.translation().xy();
            let half_size = 0.5 * node.size();
            let min = node_position - half_size;
            let max = node_position + half_size;
            (min.x..max.x).contains(&cursor_position.x)
                && (min.y..max.y).contains(&cursor_position.y)
        });
}