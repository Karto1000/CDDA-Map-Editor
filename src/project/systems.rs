use std::sync::Arc;

use bevy::prelude::{Commands, Entity, EventReader, EventWriter, NextState, Query, ResMut, With};

use crate::map::data::{ClearTiles, SpawnMapEntity};
use crate::program::data::{OpenedProject, Program, ProgramState};
use crate::project::data::{CloseProject, OpenProjectAtIndex};
use crate::ui::grid::resources::Grid;

pub fn open_project(
    mut e_open_project: EventReader<OpenProjectAtIndex>,
    mut e_clear_tiles: EventWriter<ClearTiles>,
    mut e_spawn_map_entity: EventWriter<SpawnMapEntity>,
    mut r_program: ResMut<Program>,
    mut s_next: ResMut<NextState<ProgramState>>,
    mut commands: Commands,
    q_opened_project: Query<Entity, With<OpenedProject>>,
) {
    for switch_project in e_open_project.read() {
        let new_project = r_program.projects.get(switch_project.index as usize).unwrap();

        s_next.set(ProgramState::ProjectOpen);
        
        if let Some(o) = q_opened_project.iter().next() {
            // Despawn the already existing entity
            commands.get_entity(o).unwrap().despawn();
        }
        
        commands.spawn(OpenedProject { index: switch_project.index as usize });

        e_clear_tiles.send(ClearTiles {});

        e_spawn_map_entity.send(SpawnMapEntity {
            map_entity: Arc::new(new_project.map_entity.clone())
        });
    }
}


pub fn close_project(
    mut s_next: ResMut<NextState<ProgramState>>,
    mut e_close_project: EventReader<CloseProject>,
    mut e_clear_tiles: EventWriter<ClearTiles>,
    mut r_grid: ResMut<Grid>,
    mut commands: Commands,
    q_opened_project: Query<Entity, With<OpenedProject>>,
) {
    for _ in e_close_project.read() {
        let entity = match q_opened_project.iter().next() {
            None => return,
            Some(e) => e
        };

        s_next.set(ProgramState::NoneOpen);
        commands.get_entity(entity).unwrap().despawn();

        e_clear_tiles.send(ClearTiles {});

        match r_grid.instantiated_grid {
            None => {}
            Some(g) => { commands.get_entity(g).unwrap().despawn() }
        }

        r_grid.instantiated_grid = None;
    }
}