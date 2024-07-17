use bevy::app::{App, Plugin, PostStartup, Update};
use bevy::prelude::{apply_deferred, Color, Component, in_state, IntoSystemConfigs, Resource};

use crate::program::data::ProgramState;
use crate::ui::hotbar::spawn_hotbar;
use crate::ui::interaction::{cdda_folder_picked, CDDADirPicked, close_button_interaction, file_dialog_cdda_dir_picked, file_loaded_reader, import_button_interaction, open_button_interaction, save_button_interaction, settings_button_interaction, tileset_selected, TilesetSelected};
use crate::ui::systems::{button_hover_system, button_toggle_system, check_ui_interaction, reset_toggle_reader, ResetToggle, spawn_initial_tabs};
use crate::ui::tabs::{create_project_menu, on_add_tab_button_click, setup, spawn_tab_reader, tab_clicked};
use crate::ui::tabs::events::SpawnTab;

mod systems;
pub(crate) mod interaction;
pub(crate) mod hotbar;
pub(crate) mod tabs;
pub(crate) mod grid;
pub(crate) mod style;
mod egui_utils;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, (spawn_hotbar, spawn_initial_tabs, apply_deferred, setup).chain());
        app.insert_resource(IsCursorCaptured(false));

        app.add_event::<CDDADirPicked>();
        app.add_event::<TilesetSelected>();
        app.add_event::<ResetToggle>();
        app.add_event::<SpawnTab>();

        app.add_systems(
            Update,
            (
                save_button_interaction,
                file_dialog_cdda_dir_picked,
                tileset_selected,
            ).run_if(in_state(ProgramState::ProjectOpen)),
        );

        app.add_systems(
            Update,
            (
                reset_toggle_reader,
                button_hover_system,
                button_toggle_system,
                check_ui_interaction,
                settings_button_interaction,
                cdda_folder_picked,
                close_button_interaction,
                import_button_interaction,
                open_button_interaction,
                file_loaded_reader,
                spawn_tab_reader,
                on_add_tab_button_click,
                tab_clicked,
                file_dialog_cdda_dir_picked,
                tileset_selected,
                create_project_menu
            ).chain(),
        );
    }
}

#[derive(Component)]
pub struct OriginalColor(pub Color);


#[derive(Component, Debug)]
pub struct HoverEffect {
    pub original_color: Color,
    pub hover_color: Color,
}

#[derive(Component, Debug)]
pub struct ToggleEffect {
    pub original_color: Color,
    pub toggled_color: Color,
    pub toggled: bool,
}


#[derive(Debug)]
pub struct CDDADirContents;

#[derive(Resource)]
pub struct IsCursorCaptured(pub bool);
