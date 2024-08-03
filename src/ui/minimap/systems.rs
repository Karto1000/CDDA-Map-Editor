use bevy::prelude::{Component, Entity, Query, Res, With};
use bevy::window::PrimaryWindow;
use bevy_egui::egui::{Align2, Direction, Frame, Layout, Margin, RichText, Vec2, Window};
use bevy_inspector_egui::bevy_egui::EguiContexts;
use log::warn;

use crate::common::Coordinates;
use crate::program::data::{Config, IntoColor32, OpenedProject, Program};
use crate::project::data::Project;
use crate::tiles::data::Tile;
use crate::ui::grid::resources::Grid;

#[derive(Component)]
pub struct MinimapMarker;

pub fn show_minimap(
    mut contexts: EguiContexts,
    r_program: Res<Program>,
    r_grid: Res<Grid>,
    q_windows: Query<&bevy::prelude::Window, With<PrimaryWindow>>,
    q_opened_project: Query<(Entity, &OpenedProject)>,
) {
    let opened_project = match q_opened_project.iter().next() {
        None => return,
        Some(o) => o.1
    };
    
    let window = q_windows.single();
    
    Window::new("")
        .resizable(false)
        .anchor(Align2::RIGHT_BOTTOM, Vec2::new(-8., -8.))
        .collapsible(false)
        .title_bar(false)
        .min_size(Vec2::new(200., 200.))
        .max_size(Vec2::new(200., 200.))
        .show(contexts.ctx_mut(), |ui| {
            ui.set_width(200.);
            ui.set_height(200.);

            let project = match r_program.projects.get(opened_project.index) {
                None => {
                    warn!("Failed to get Project at index {}. Project does not exist", opened_project.index);
                    return;
                }
                Some(p) => p
            };

            let tile_camera_offset = r_grid.offset / r_grid.tile_size;
            
            let window_tile_center_w = (window.resolution.width() / r_grid.tile_size) / 2.;
            let window_tile_center_h = (window.resolution.height() / r_grid.tile_size) / 2.;

            ui.vertical(|ui| {
                for y in (tile_camera_offset.y + window_tile_center_h) as i32.. (16. + tile_camera_offset.y + window_tile_center_h) as i32 {
                    let mut row: String = "".into();

                    for x in (tile_camera_offset.x + window_tile_center_w) as i32..(16. + tile_camera_offset.x + window_tile_center_w) as i32 {
                        let tile = match project.map_entity.tiles().get(&Coordinates::new(x, y)) {
                            None => {
                                row.push_str(" ");
                                continue;
                            }
                            Some(t) => t
                        };

                        row.push(tile.character);
                    }

                    ui.label(RichText::new(row).size(12.).extra_letter_spacing(6.));
                }
            });
        });
}