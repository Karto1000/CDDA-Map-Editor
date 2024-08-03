use std::collections::HashMap;

use bevy::asset::AssetServer;
use bevy::hierarchy::BuildChildren;
use bevy::prelude::{AlignContent, BackgroundColor, ButtonBundle, Changed, Color, Commands, default, Display, Entity, EventReader, EventWriter, ImageBundle, Interaction, IVec2, NodeBundle, Query, Res, ResMut, Resource, State, Style, Text, TextBundle, TextStyle, UiImage, UiRect, Val, With};
use bevy::utils::petgraph::visit::Walker;
use bevy_egui::egui::{Align2, Button, Vec2, Window};
use bevy_inspector_egui::bevy_egui::EguiContexts;

use crate::common::Coordinates;
use crate::map::data::{MapEntity, Single};
use crate::program::data::{IntoColor32, OpenedProject, Program, ProgramState};
use crate::program::data::Menus;
use crate::project::data::{CloseProject, CreateProject, Project};
use crate::project::data::OpenProjectAtIndex;
use crate::tiles::data::Tile;
use crate::ui::{HoverEffect, ToggleEffect};
use crate::ui::egui_utils::{add_settings_frame, input_group};
use crate::ui::hotbar::components::TopHotbarMarker;
use crate::ui::tabs::components::{AddTabButtonMarker, Tab, TabContainerMarker};
use crate::ui::tabs::events::SpawnTab;

pub(crate) mod events;
pub(crate) mod components;

#[derive(Resource, Debug, Default)]
pub struct CreateData {
    name: String,
    size: String,
}

pub fn setup(
    r_asset_server: Res<AssetServer>,
    q_top_hotbar: Query<Entity, With<TopHotbarMarker>>,
    r_program: Res<Program>,
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
                        background_color: BackgroundColor::from(r_program.config.style.gray_dark),
                        ..default()
                    },
                    HoverEffect { original_color: r_program.config.style.gray_dark, hover_color: r_program.config.style.gray_light },
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

pub fn create_project_menu(
    mut contexts: EguiContexts,
    mut r_program: ResMut<Program>,
    mut r_menus: ResMut<Menus>,
    mut r_create_data: Option<ResMut<CreateData>>,
    mut e_spawn_tab: EventWriter<SpawnTab>,
    mut e_create_project: EventWriter<CreateProject>,
    mut commands: Commands,
) {
    let mut r_create_data = match r_create_data {
        None => return,
        Some(r) => r
    };

    let gray_dark_color32 = r_program.config.style.gray_dark.into_color32();
    let amount_of_projects = r_program.projects.len();

    Window::new("Create new Project")
        .open(&mut r_menus.is_create_project_menu_open)
        .collapsible(false)
        .resizable(false)
        .movable(false)
        .anchor(Align2::CENTER_CENTER, Vec2::default())
        .show(contexts.ctx_mut(), |ui| {
            ui.set_max_width(500.);

            add_settings_frame(
                "Config",
                gray_dark_color32,
                ui,
                |ui| {
                    input_group(
                        ui,
                        &mut r_create_data.name,
                        "Name".into(),
                    );

                    input_group(
                        ui,
                        &mut r_create_data.size,
                        "Map Size".into(),
                    );
                },
            );

            let button = Button::new("Create");
            let response = ui.add_sized([64., 32.], button);

            if response.clicked() {
                if r_create_data.name.is_empty() { return; }
                if r_create_data.size.is_empty() { return; }

                let nums = r_create_data.size.splitn(2, "x").collect::<Vec<&str>>();

                let map_size: IVec2 = match (nums.get(0), nums.get(1)) {
                    (Some(width), Some(height)) => {
                        match (width.parse(), height.parse()) {
                            (Ok(v1), Ok(v2)) => IVec2::new(v1, v2),
                            _ => return
                        }
                    }
                    _ => return
                };

                let mut default_tiles = HashMap::new();

                for y in 0..map_size.y {
                    for x in 0..map_size.x {
                        default_tiles.insert(
                            Coordinates::new(x, y),
                            Tile::from(' '),
                        );
                    }
                }

                let project = Project {
                    name: r_create_data.name.clone(),
                    map_entity: MapEntity::Single(Single {
                        om_terrain: r_create_data.name.clone(),
                        tile_selection: Default::default(),
                        tiles: default_tiles,
                        size: map_size,
                    }),
                    save_state: Default::default(),
                };

                e_create_project.send(CreateProject {
                    project
                });

                e_spawn_tab.send(SpawnTab {
                    name: r_create_data.name.clone(),
                    index: amount_of_projects as u32,
                });

                commands.remove_resource::<CreateData>();
            }
        });
}

pub fn on_add_tab_button_click(
    q_interaction: Query<&Interaction, (Changed<Interaction>, With<AddTabButtonMarker>)>,
    mut r_menus: ResMut<Menus>,
    mut commands: Commands,
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
                    r_menus.is_create_project_menu_open = !r_menus.is_create_project_menu_open;
                    commands.insert_resource(CreateData::default())
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
    r_program: Res<Program>,
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
                    background_color: BackgroundColor::from(r_program.config.style.blue_dark),
                    ..default()
                },
                HoverEffect {
                    original_color: r_program.config.style.blue_dark,
                    hover_color: r_program.config.style.selected,
                },
                ToggleEffect {
                    original_color: r_program.config.style.blue_dark,
                    toggled_color: r_program.config.style.selected,
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
    mut e_open_project: EventWriter<OpenProjectAtIndex>,
    mut e_close_project: EventWriter<CloseProject>,
    mut q_interaction: Query<(&Interaction, &Tab), (Changed<Interaction>, With<Tab>)>,
    r_program: Res<Program>,
    s_state: Res<State<ProgramState>>,
    q_opened_project: Query<(Entity, &OpenedProject)>,
) {
    for (interaction, tab) in q_interaction.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                match s_state.get() {
                    ProgramState::ProjectOpen => {
                        let index = match q_opened_project.iter().next() {
                            None => return,
                            Some(o) => o.1.index
                        };

                        if tab.index == index as u32 {
                            e_close_project.send(CloseProject {});
                            return;
                        }
                    }
                    ProgramState::NoneOpen => {}
                };

                match r_program.projects.get(tab.index as usize) {
                    None => { return; }
                    Some(_) => {}
                };

                e_open_project.send(OpenProjectAtIndex {
                    index: tab.index
                });
            }
            _ => {}
        }
    };
}