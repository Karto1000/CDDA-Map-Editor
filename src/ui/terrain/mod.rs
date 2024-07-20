use bevy::prelude::{AssetServer, Query, Res, ResMut, Resource};
use bevy_egui::egui::{Align, Button, Color32, Frame, Layout, Margin, RichText, ScrollArea, TextureId, Ui, Vec2, Window};
use bevy_egui::egui::load::SizedTexture;
use bevy_inspector_egui::bevy_egui::EguiContexts;

use crate::common::{MeabyWeighted, TileId};
use crate::graphics::GraphicsResource;
use crate::palettes::data::{MapGenValue, MapObjectId, MeabyParam};
use crate::program::data::{IntoColor32, Menus, OpenedProject, Program};

#[derive(Resource, Default)]
pub struct TerrainMenuData {
    search_text: String,
}

fn add_single_tile(
    repr: SingleTileRepr,
    ui: &mut Ui,
    r_program: &Program,
) {
    Frame::none()
        .fill(r_program.config.style.gray_dark.into_color32())
        .inner_margin(Margin::same(4.))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());

            ui.horizontal(|ui| {
                ui.horizontal(|ui| {
                    Frame::none()
                        .fill(r_program.config.style.selected.into_color32())
                        .show(ui, |ui| {
                            ui.set_min_size(Vec2::new(32., 32.));
                            ui.set_max_size(Vec2::new(32., 32.));

                            ui.centered_and_justified(|ui| {
                                ui.label(RichText::new(format!("{}", repr.char)).size(16.))
                            });
                        });

                    if let Some(fg) = repr.fg {
                        ui.image(SizedTexture::new(fg, Vec2::new(32., 32.)));
                    }

                    ui.label(RichText::new(repr.id).size(16.))
                });

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.add_sized(
                        Vec2::new(32., 32.),
                        Button::new("X").fill(r_program.config.style.error.into_color32()),
                    );
                });
            });
        });
}

fn add_parameter(
    repr: ParameterRepr,
    ui: &mut Ui,
    r_program: &Program,
) {
    Frame::none()
        .fill(r_program.config.style.gray_dark.into_color32())
        .inner_margin(Margin::same(2.))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());

            Frame::none()
                .fill(r_program.config.style.selected.into_color32())
                .inner_margin(Margin {
                    left: 14.0,
                    right: 4.0,
                    top: 4.0,
                    bottom: 4.0,
                })
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.set_width(ui.available_width());
                        ui.set_height(32.);

                        ui.horizontal_centered(|ui| {
                            ui.label(RichText::new(repr.name).size(16.));
                        });

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.add_sized(
                                Vec2::new(32., 32.),
                                Button::new("X").fill(r_program.config.style.error.into_color32()),
                            );

                            ui.add_sized(
                                Vec2::new(32., 32.),
                                Button::new("+").fill(r_program.config.style.blue_dark.into_color32()),
                            );
                        });
                    });
                });


            let total_weight: u32 = repr.distribution.iter()
                .map(|(mw, _, _)| {
                    match mw {
                        MeabyWeighted::NotWeighted(_) => 1,
                        MeabyWeighted::Weighted(w) => w.weight
                    }
                })
                .sum();

            for (tile, fg_id, bg_id) in repr.distribution {
                let weight = match &tile {
                    MeabyWeighted::NotWeighted(_) => 1,
                    MeabyWeighted::Weighted(w) => w.weight
                };

                Frame::none()
                    .fill(r_program.config.style.gray_dark.into_color32())
                    .inner_margin(Margin {
                        left: 0.0,
                        right: 4.0,
                        top: 0.0,
                        bottom: 0.0,
                    })
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.set_width(ui.available_width());
                            ui.set_height(32.);

                            Frame::none()
                                .fill(Color32::from_hex("#79FD8E").unwrap())
                                .show(ui, |ui| {
                                    ui.set_width(2.);
                                    ui.set_height(ui.available_height());
                                });

                            if let Some(fg_id) = fg_id {
                                ui.image(
                                    SizedTexture::new(
                                        fg_id,
                                        Vec2::new(32., 32.),
                                    )
                                );
                            }

                            ui.label(RichText::new(tile.value()).size(16.));

                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.add_sized(
                                    Vec2::new(32., 32.),
                                    Button::new("X").fill(r_program.config.style.error.into_color32()),
                                );
                                ui.label(RichText::new(format!("{:.2}%", (weight as f32 / total_weight as f32) * 100.)).size(16.));
                            });
                        });
                    });
            }
        });
}

fn add_grouped_tile(
    repr: GroupedTileRepr,
    ui: &mut Ui,
    r_program: &Program,
) {
    Frame::none()
        .fill(r_program.config.style.gray_dark.into_color32())
        .inner_margin(Margin::same(2.))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());

            Frame::none()
                .fill(r_program.config.style.selected.into_color32())
                .inner_margin(Margin {
                    left: 14.0,
                    right: 4.0,
                    top: 4.0,
                    bottom: 4.0,
                })
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.set_width(ui.available_width());
                        ui.set_height(32.);
                        ui.horizontal_centered(|ui| {
                            ui.label(RichText::new(repr.char).size(16.));
                        });

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.add_sized(
                                Vec2::new(32., 32.),
                                Button::new("X").fill(r_program.config.style.error.into_color32()),
                            );

                            ui.add_sized(
                                Vec2::new(32., 32.),
                                Button::new("+").fill(r_program.config.style.blue_dark.into_color32()),
                            );
                        });
                    });
                });


            let total_weight: u32 = repr.distribution.iter()
                .map(|(mw, _, _)| {
                    match mw {
                        MeabyWeighted::NotWeighted(_) => 1,
                        MeabyWeighted::Weighted(w) => w.weight
                    }
                })
                .sum();

            for (tile, fg_id, bg_id) in repr.distribution {
                let weight = match &tile {
                    MeabyWeighted::NotWeighted(_) => 1,
                    MeabyWeighted::Weighted(w) => w.weight
                };

                Frame::none()
                    .fill(r_program.config.style.gray_dark.into_color32())
                    .inner_margin(Margin {
                        left: 0.0,
                        right: 4.0,
                        top: 0.0,
                        bottom: 0.0,
                    })
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.set_width(ui.available_width());
                            ui.set_height(32.);

                            Frame::none()
                                .fill(Color32::from_hex("#79FD8E").unwrap())
                                .show(ui, |ui| {
                                    ui.set_width(2.);
                                    ui.set_height(ui.available_height());
                                });

                            match tile.value() {
                                MeabyParam::TileId(id) => {
                                    if let Some(fg_id) = fg_id {
                                        ui.image(
                                            SizedTexture::new(
                                                fg_id,
                                                Vec2::new(32., 32.),
                                            )
                                        );
                                    }

                                    ui.label(RichText::new(id).size(16.));
                                }
                                MeabyParam::Parameter(_) => {}
                            }

                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.add_sized(
                                    Vec2::new(32., 32.),
                                    Button::new("X").fill(r_program.config.style.error.into_color32()),
                                );
                                ui.label(RichText::new(format!("{:.2}%", (weight as f32 / total_weight as f32) * 100.)).size(16.));
                            });
                        });
                    });
            }
        });
}

struct ParameterRepr {
    name: String,

    distribution: Vec<(MeabyWeighted<String>, Option<TextureId>, Option<TextureId>)>,
}

struct SingleTileRepr {
    char: char,
    id: TileId,
    fg: Option<TextureId>,
    bg: Option<TextureId>,
}

struct GroupedTileRepr {
    char: char,
    distribution: Vec<(MeabyWeighted<MeabyParam>, Option<TextureId>, Option<TextureId>)>,
}

pub fn terrain_menu(
    mut contexts: EguiContexts,
    mut r_program: ResMut<Program>,
    mut r_menus: ResMut<Menus>,
    r_graphics: Res<GraphicsResource>,
    r_asset_server: Res<AssetServer>,
    q_open_project: Query<&OpenedProject>,
    r_terrain_menu_data: Option<ResMut<TerrainMenuData>>,
) {
    let mut terrain_menu_data = match r_terrain_menu_data {
        None => return,
        Some(d) => d
    };

    let opened_project = match q_open_project.iter().next() {
        None => return,
        Some(p) => p
    };

    let textures = match &r_graphics.textures {
        None => return,
        Some(t) => t
    };

    let project = r_program.projects.get(opened_project.index).unwrap();

    let terrain = &project
        .map_entity
        .object()
        .terrain;

    let mut single_tiles: Vec<SingleTileRepr> = vec![];
    let mut grouped_tiles: Vec<GroupedTileRepr> = vec![];
    let mut parameters: Vec<ParameterRepr> = vec![];

    project.map_entity.object().parameters.iter()
        .for_each(|(name, parameter)| {
            let distribution = match &parameter.default {
                MapGenValue::Distribution { distribution } => {
                    distribution.iter().map(|mw| {
                        let repr = textures.get_terrain_representation(mw.value());

                        (
                            mw.clone(),
                            repr.fg.as_ref()
                                .map(|v| contexts.add_image(v.get_representative_sprite().clone_weak())),
                            repr.bg.as_ref()
                                .map(|v| contexts.add_image(v.get_representative_sprite().clone_weak())),
                        )
                    }).collect()
                }
                _ => todo!()
            };

            parameters.push(ParameterRepr {
                name: name.clone(),
                distribution,
            })
        });

    terrain.iter()
        .for_each(|(char, object_id)| {
            match object_id {
                MapObjectId::Single(s) => {
                    match s.value() {
                        MeabyParam::TileId(id) => {
                            let repr = textures.get_terrain_representation(id);

                            let fg_sprite = repr.fg.as_ref()
                                .map(|v| contexts.add_image(v.get_representative_sprite().clone_weak()));
                            let bg_sprite = repr.bg.as_ref()
                                .map(|v| contexts.add_image(v.get_representative_sprite().clone_weak()));

                            single_tiles.push(SingleTileRepr {
                                id: id.clone(),
                                fg: fg_sprite,
                                bg: bg_sprite,
                                char: char.clone(),
                            });
                        }
                        MeabyParam::Parameter(_) => todo!()
                    }
                }
                MapObjectId::Grouped(v) => {
                    let distribution: Vec<(MeabyWeighted<MeabyParam>, Option<TextureId>, Option<TextureId>)> = v.iter().map(|mw| {
                        match mw.value() {
                            MeabyParam::TileId(id) => {
                                let repr = textures.get_terrain_representation(id);

                                (
                                    mw.clone(),
                                    repr.fg.as_ref()
                                        .map(|v| contexts.add_image(v.get_representative_sprite().clone_weak())),
                                    repr.bg.as_ref()
                                        .map(|v| contexts.add_image(v.get_representative_sprite().clone_weak())),
                                )
                            }
                            MeabyParam::Parameter(_) => todo!()
                        }
                    }).collect();

                    grouped_tiles.push(GroupedTileRepr {
                        char: char.clone(),
                        distribution,
                    })
                }
                MapObjectId::Param { param, .. } => {
                    let fg_sprite = contexts.add_image(r_asset_server.load("parameter.png"));

                    single_tiles.push(SingleTileRepr {
                        char: char.clone(),
                        id: param.clone(),
                        fg: Some(fg_sprite),
                        bg: None,
                    })
                }
                _ => return,
            };
        });

    Window::new("Define new Terrain")
        .open(&mut r_menus.is_define_terrain_menu_open)
        .resizable(true)
        .show(contexts.ctx_mut(), |ui| {
            ui.vertical(|ui| {
                ui.label(
                    "Here you you can define a list of tiles which will be inserted into the \
                    terrain field in the generated MapGen tile without having to specify a new Palette."
                );

                ui.horizontal(|ui| {
                    ui.set_height(32.);

                    ui.with_layout(
                        Layout::right_to_left(Align::Center),
                        |ui| {
                            ui.button("+");
                            ui.text_edit_singleline(&mut terrain_menu_data.search_text);
                        },
                    );
                });

                ui.vertical(|ui| {
                    ui.label(RichText::new("Parameters").size(16.));

                    for parameter in parameters {
                        add_parameter(
                            parameter,
                            ui,
                            r_program.as_ref(),
                        );
                    }

                    ui.label(RichText::new("Single Tiles").size(16.));

                    ScrollArea::vertical()
                        .show(ui, |ui| {
                            for tile in single_tiles {
                                add_single_tile(
                                    tile,
                                    ui,
                                    r_program.as_ref(),
                                );
                            }
                        });

                    ui.label(RichText::new("Nested Tiles").size(16.));

                    for tiles in grouped_tiles {
                        add_grouped_tile(
                            tiles,
                            ui,
                            r_program.as_ref(),
                        )
                    }
                });
            });
        });
}