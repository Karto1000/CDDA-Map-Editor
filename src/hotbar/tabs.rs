use bevy::asset::AssetServer;
use bevy::hierarchy::BuildChildren;
use bevy::log::warn;
use bevy::prelude::{AlignContent, BackgroundColor, ButtonBundle, Changed, Color, Commands, default, Display, Entity, Event, EventReader, EventWriter, FlexDirection, ImageBundle, Interaction, NodeBundle, Query, Res, Style, Text, TextBundle, TextStyle, UiImage, UiRect, Val, With};

use crate::hotbar::systems::{AddTabButtonMarker, PRIMARY_COLOR_FADED, TabContainerMarker, TopHotbarMarker};
use crate::project::Project;

#[derive(Event)]
pub struct SpawnTab {
    pub project: Project,
}

pub fn setup(
    r_asset_server: Res<AssetServer>,
    q_top_hotbar: Query<Entity, With<TopHotbarMarker>>,
    mut commands: Commands,
) {
    let hotbar = match q_top_hotbar.iter().next() {
        None => {
            warn!("No Hotbar");
            return;
        }
        Some(h) => h
    };

    let mut entity = commands.get_entity(hotbar).unwrap();
    entity.with_children(|parent| {
        parent.spawn((
            NodeBundle {
                style: Style {
                    width: Val::Auto,
                    height: Val::Px(32.),
                    display: Display::Flex,
                    flex_direction: FlexDirection::RowReverse,
                    ..default()
                },
                ..default()
            },
            TabContainerMarker {}
        )).with_children(|parent| {
            parent.spawn((
                ButtonBundle {
                    style: Style {
                        width: Val::Px(32.),
                        height: Val::Px(32.),
                        ..default()
                    },
                    background_color: BackgroundColor::from(PRIMARY_COLOR_FADED),
                    ..default()
                },
                crate::hotbar::systems::OriginalColor { 0: PRIMARY_COLOR_FADED },
                AddTabButtonMarker {}
            )).with_children(|parent| {
                parent.spawn(
                    ImageBundle {
                        style: Style {
                            width: Val::Px(10.),
                            height: Val::Px(10.),
                            margin: UiRect::all(Val::Auto),
                            ..default()
                        },
                        image: UiImage::new(r_asset_server.load("icons/add.png")),
                        ..default()
                    }
                );
            });
        });
    });
}

pub fn on_add_tab_button_click(
    q_interaction: Query<(&Interaction), (Changed<Interaction>, With<AddTabButtonMarker>)>,
    mut e_spawn_tab: EventWriter<SpawnTab>,
) {
    let interaction = match q_interaction.iter().next() {
        None => { return; }
        Some(i) => i
    };

    match interaction {
        Interaction::Pressed => {
            e_spawn_tab.send(SpawnTab { project: Project::default() })
        }
        _ => {}
    }
}

pub fn spawn_tab_reader(
    mut e_spawn_tab: EventReader<SpawnTab>,
    top_hotbar: Query<(Entity), With<TabContainerMarker>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    for event in e_spawn_tab.read() {
        let mut entity_commands = commands.get_entity(top_hotbar.single()).unwrap();
        entity_commands.with_children(|parent| {
            parent.spawn(ButtonBundle {
                style: Style {
                    display: Display::Flex,
                    column_gap: Val::Px(12.),
                    height: Val::Px(32.),
                    width: Val::Auto,
                    align_content: AlignContent::Center,
                    padding: UiRect::px(9., 9., 8., 8.),
                    ..default()
                },
                background_color: BackgroundColor::from(PRIMARY_COLOR_FADED),
                ..default()
            }).with_children(|parent| {
                parent.spawn(TextBundle {
                    text: Text::from_section(
                        event.project.map_entity.name.clone(),
                        TextStyle {
                            font: asset_server.load("fonts/unifont.ttf"),
                            font_size: 12.,
                            color: Color::hex("#FFFFFF").unwrap(),
                        },
                    ),
                    ..default()
                });

                parent.spawn(ButtonBundle {
                    style: Style {
                        height: Val::Percent(100.),
                        margin: UiRect::axes(Val::Px(0.), Val::Px(3.)),
                        aspect_ratio: Some(1.),
                        ..default()
                    },
                    background_color: BackgroundColor::from(Color::rgba(1., 1., 1., 0.)),
                    ..default()
                }).with_children(|parent| {
                    parent.spawn(ImageBundle {
                        style: Style {
                            width: Val::Px(10.),
                            height: Val::Px(10.),
                            ..default()
                        },
                        image: UiImage::new(asset_server.load("icons/close.png")),
                        ..default()
                    });
                });
            });
        }
        );
    };
}