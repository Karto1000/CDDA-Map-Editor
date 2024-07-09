use bevy::prelude::{EventReader, EventWriter, ResMut};
use std::sync::Arc;
use crate::editor_data::data::EditorData;
use crate::map::data::{ClearTiles, SpawnMapEntity};
use crate::project::data::SwitchProject;

pub fn switch_project(
    mut e_switch_project: EventReader<SwitchProject>,
    mut e_clear_tiles: EventWriter<ClearTiles>,
    mut e_spawn_map_entity: EventWriter<SpawnMapEntity>,
    mut r_editor_data: ResMut<EditorData>,
) {
    for switch_project in e_switch_project.read() {
        let new_project = r_editor_data.projects.get(switch_project.index as usize).unwrap();

        e_clear_tiles.send(ClearTiles {});

        e_spawn_map_entity.send(SpawnMapEntity {
            map_entity: Arc::new(new_project.map_entity.clone())
        });

        r_editor_data.current_project_index = Some(switch_project.index);
    }
}
