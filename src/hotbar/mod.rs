use bevy::app::{App, Plugin, Startup, Update};
use bevy::prelude::{apply_deferred, IntoSystemConfigs};

use crate::hotbar::interaction::{close_button_interaction, file_loaded_reader, import_button_interaction, open_button_interaction, save_button_interaction};
use crate::hotbar::systems::{button_hover_system, button_toggle_system, check_ui_interaction, reset_toggle_reader, ResetToggle, spawn_hotbar};
use crate::hotbar::tabs::{on_add_tab_button_click, setup, spawn_tab_reader, SpawnTab, tab_clicked};

mod systems;
mod interaction;
pub(crate) mod tabs;

pub struct HotbarPlugin;

impl Plugin for HotbarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_hotbar, apply_deferred, setup).chain());

        app.add_event::<ResetToggle>();
        app.add_systems(Update, reset_toggle_reader);
        app.add_systems(Update, button_hover_system);
        app.add_systems(Update, button_toggle_system);
        app.add_systems(Update, check_ui_interaction);

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