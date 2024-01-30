use bevy::app::{App, Plugin, Startup, Update};
use crate::hotbar::interaction::{close_button_interaction, import_button_interaction, save_button_interaction};

use crate::hotbar::systems::{button_system, check_ui_interaction, spawn_hotbar};

mod systems;
mod interaction;

pub struct HotbarPlugin;

impl Plugin for HotbarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_hotbar);
        app.add_systems(Update, button_system);
        app.add_systems(Update, check_ui_interaction);

        // Hotbar Button interactions
        app.add_systems(Update, close_button_interaction);
        app.add_systems(Update, save_button_interaction);
        app.add_systems(Update, import_button_interaction);
    }
}