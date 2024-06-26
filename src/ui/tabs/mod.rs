use bevy::asset::AssetServer;
use bevy::hierarchy::BuildChildren;
use bevy::prelude::{AlignContent, BackgroundColor, ButtonBundle, Changed, Color, Commands, default, Display, Entity, EventReader, EventWriter, ImageBundle, Interaction, NodeBundle, Query, Res, ResMut, Style, Text, TextBundle, TextStyle, UiImage, UiRect, Val, With};

use crate::editor_data::EditorData;
use crate::map::resources::MapEntity;
use crate::project::resources::{Project, ProjectSaveState};
use crate::SwitchProject;
use crate::ui::components::{HoverEffect, ToggleEffect};
use crate::ui::hotbar::components::TopHotbarMarker;
use crate::ui::tabs::components::{AddTabButtonMarker, Tab, TabContainerMarker};
use crate::ui::tabs::events::SpawnTab;

pub(crate) mod events;
pub(crate) mod components;

pub fn setup(
    r_asset_server: Res<AssetServer>,
    q_top_hotbar: Query<Entity, With<TopHotbarMarker>>,
    r_editor_data: Res<EditorData>,
    mut commands: Commands,
) {
    let hotbar = q_top_hotbar.iter().next().unwrap();

    let mut entity = commands.get_entity(hotbar).unwrap();
    entity.with_children(|parent| {
        parent.spawn((
            NodeBundle {
                style: Style {
                    width: Val::Auto,
                    height: Val::Px(32.),
                    display: Display::Flex,
                    ..default()
                },
                ..default()
            },
        ))
            .with_children(|parent| {
                parent.spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Auto,
                            height: Val::Px(32.),
                            display: Display::Flex,
                            ..default()
                        },
                        ..default()
                    },
                    TabContainerMarker {}
                ));
            })
            .with_children(|parent| {
                parent.spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(32.),
                            height: Val::Px(32.),
                            ..default()
                        },
                        background_color: BackgroundColor::from(r_editor_data.config.style.gray_dark),
                        ..default()
                    },
                    HoverEffect { original_color: r_editor_data.config.style.gray_dark, hover_color: r_editor_data.config.style.gray_light },
                    AddTabButtonMarker {},
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
    q_interaction: Query<&Interaction, (Changed<Interaction>, With<AddTabButtonMarker>)>,
    mut r_editor_data: ResMut<EditorData>,
    mut e_spawn_tab: EventWriter<SpawnTab>,
) {
    let interaction = match q_interaction.iter().next() {
        None => { return; }
        Some(i) => i
    };

    match interaction {
        Interaction::Pressed => {
            let mut project = Project::default();

            match project.map_entity {
                MapEntity::Single(ref mut s) => {
                    let amount_of_unnamed = r_editor_data.projects.iter()
                        .filter(|p| p.save_state == ProjectSaveState::NotSaved)
                        .map(|p| s.om_terrain.clone())
                        .filter(|n| n.contains("unnamed"))
                        .count();

                    s.om_terrain = format!("unnamed{}", amount_of_unnamed).to_string();

                    let name = s.om_terrain.clone();

                    r_editor_data.projects.push(project);
                    e_spawn_tab.send(SpawnTab { name, index: r_editor_data.projects.len() as u32 - 1 });
                }
                _ => todo!()
            }
        }
        _ => {}
    }
}

pub fn spawn_tab_reader(
    top_hotbar: Query<Entity, With<TabContainerMarker>>,
    asset_server: Res<AssetServer>,
    r_editor_data: Res<EditorData>,
    mut e_spawn_tab: EventReader<SpawnTab>,
    mut commands: Commands,
) {
    for event in e_spawn_tab.read() {
        let mut entity_commands = commands.get_entity(top_hotbar.single()).unwrap();

        entity_commands.with_children(|parent| {
            parent.spawn((
                ButtonBundle {
                    style: Style {
                        display: Display::Flex,
                        column_gap: Val::Px(12.),
                        height: Val::Px(32.),
                        width: Val::Auto,
                        align_content: AlignContent::Center,
                        padding: UiRect::px(9., 9., 8., 8.),
                        ..default()
                    },
                    background_color: BackgroundColor::from(r_editor_data.config.style.blue_dark),
                    ..default()
                },
                HoverEffect {
                    original_color: r_editor_data.config.style.blue_dark,
                    hover_color: r_editor_data.config.style.selected,
                },
                ToggleEffect {
                    original_color: r_editor_data.config.style.blue_dark,
                    toggled_color: r_editor_data.config.style.selected,
                    toggled: false,
                },
                Tab { index: event.index }
            )).with_children(|parent| {
                parent.spawn(TextBundle {
                    text: Text::from_section(
                        event.name.clone(),
                        TextStyle {
                            font: asset_server.load("fonts/unifont.ttf"),
                            font_size: 12.,
                            color: Color::hex("#FFFFFF").unwrap(),
                        },
                    ),
                    ..default()
                });

                parent.spawn((
                    ButtonBundle {
                        style: Style {
                            height: Val::Auto,
                            width: Val::Auto,
                            margin: UiRect::axes(Val::Px(0.), Val::Px(3.)),
                            aspect_ratio: Some(1.),
                            ..default()
                        },
                        background_color: BackgroundColor::from(Color::rgba(1., 1., 1., 0.)),
                        ..default()
                    },
                )).with_children(|parent| {
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

pub fn tab_clicked(
    mut e_switch_project: EventWriter<SwitchProject>,
    r_editor_data: Res<EditorData>,
    mut q_interaction: Query<(&Interaction, &Tab), (Changed<Interaction>, With<Tab>)>,
) {
    for (interaction, tab) in q_interaction.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                match r_editor_data.projects.get(tab.index as usize) {
                    None => { return; }
                    Some(_) => {}
                };

                e_switch_project.send(SwitchProject {
                    index: tab.index
                });
            }
            _ => {}
        }
    };
}