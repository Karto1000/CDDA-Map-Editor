use bevy::app::{App, Plugin, PostStartup, Update};
use bevy::prelude::{apply_deferred, IntoSystemConfigs};

use crate::ui::hotbar::spawn_hotbar;
use crate::ui::interaction::{cdda_folder_picked, CDDADirPicked, close_button_interaction, file_loaded_reader, import_button_interaction, open_button_interaction, save_button_interaction, settings_button_interaction, TilesetSelected};
use crate::ui::systems::{button_hover_system, button_toggle_system, check_ui_interaction, reset_toggle_reader, ResetToggle};
use crate::ui::tabs::{on_add_tab_button_click, setup, spawn_tab_reader, tab_clicked};
use crate::ui::tabs::events::SpawnTab;

mod systems;
pub(crate) mod interaction;
pub(crate) mod components;
pub(crate) mod hotbar;
pub(crate) mod tabs;
pub(crate) mod grid;
pub(crate) mod style;
pub(crate) mod settings;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, (spawn_hotbar, apply_deferred, setup).chain());

        app.add_event::<CDDADirPicked>();
        app.add_event::<TilesetSelected>();
        app.add_event::<ResetToggle>();
        
        app.add_systems(Update, reset_toggle_reader);
        app.add_systems(Update, button_hover_system);
        app.add_systems(Update, button_toggle_system);
        app.add_systems(Update, check_ui_interaction);
        app.add_systems(Update, settings_button_interaction);
        app.add_systems(Update, cdda_folder_picked);

        // Hotbar Button interactions
        app.add_systems(Update, close_button_interaction);
        app.add_systems(Update, save_button_interaction);
        app.add_systems(Update, import_button_interaction);
        app.add_systems(Update, open_button_interaction);
        app.add_systems(Update, file_loaded_reader);

        // Tabs
        app.add_event::<SpawnTab>();
        app.add_systems(Update, spawn_tab_reader);
        app.add_systems(Update, on_add_tab_button_click);
        app.add_systems(Update, tab_clicked);
    }
}