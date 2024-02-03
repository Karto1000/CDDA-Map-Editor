use bevy::asset::AssetServer;
use bevy::hierarchy::BuildChildren;
use bevy::prelude::{AlignContent, BackgroundColor, ButtonBundle, Color, Commands, default, Display, Entity, Event, EventReader, ImageBundle, Query, Res, Style, Text, TextBundle, TextStyle, UiImage, UiRect, Val, With};

use crate::hotbar::systems::{PRIMARY_COLOR_SELECTED, TopHotbarMarker};
use crate::project::Project;

#[derive(Event)]
pub struct SpawnTab {
    pub project: Project,
}

pub fn spawn_tab_reader(
    mut e_spawn_tab: EventReader<SpawnTab>,
    top_hotbar: Query<(Entity), With<TopHotbarMarker>>,
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
                background_color: BackgroundColor::from(PRIMARY_COLOR_SELECTED),
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