use bevy::app::App;
use bevy::prelude::{in_state, IntoSystemConfigs, Plugin, Update};

use crate::program::data::ProgramState;
use crate::project::data::{CloseProject, OpenProjectAtIndex};
use crate::project::systems::{close_project, open_project};

pub struct ProjectPlugin;

impl Plugin for ProjectPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<OpenProjectAtIndex>();
        app.add_event::<CloseProject>();

        app.add_systems(
            Update,
            (
                open_project,
                close_project
            ).run_if(in_state(ProgramState::NoneOpen)),
        );

        app.add_systems(
            Update,
            (
                open_project,
                close_project
            ).run_if(in_state(ProgramState::ProjectOpen)),
        );
    }
}