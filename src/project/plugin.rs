use bevy::app::App;
use bevy::prelude::{apply_deferred, in_state, IntoSystemConfigs, Plugin, Update};
use crate::map::systems::spawn_map_entity_reader;
use crate::program::data::ProgramState;
use crate::project::data::{CloseProject, CreateProject, OpenProjectAtIndex};
use crate::project::systems::{close_project, create_project, open_project};

pub struct ProjectPlugin;

impl Plugin for ProjectPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<OpenProjectAtIndex>();
        app.add_event::<CloseProject>();
        app.add_event::<CreateProject>();

        app.add_systems(
            Update,
            (
                open_project,
                close_project,
                create_project
            ).chain(),
        );
    }
}