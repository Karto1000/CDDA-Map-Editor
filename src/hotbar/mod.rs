use bevy::app::{App, Plugin, Startup, Update};

use crate::hotbar::systems::{button_system, check_ui_interaction, spawn_hotbar};

mod systems;

pub struct HotbarPlugin;

impl Plugin for HotbarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_hotbar);
        app.add_systems(Update, button_system);
        app.add_systems(Update, check_ui_interaction);
    }
}