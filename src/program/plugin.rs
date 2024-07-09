use bevy::app::App;
use bevy::prelude::Plugin;
use crate::common::io::Load;
use crate::program::data::{ProgramState};
use crate::program::io::ProgramdataLoader;

pub struct ProgramPlugin;

impl Plugin for ProgramPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ProgramState>();
    }
}