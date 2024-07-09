use bevy::app::{App, Plugin, Update};
use bevy::prelude::{in_state, IntoSystemConfigs};

use crate::map::data::{ClearTiles, SpawnMapEntity, TileDeleteEvent, TilePlaceEvent, UpdateSpriteEvent};
use crate::map::systems::{clear_tiles_reader, spawn_map_entity_reader, SpawnSprite, update_animated_sprites};
use crate::program::data::ProgramState;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update, (
                spawn_map_entity_reader,
                clear_tiles_reader,
                update_animated_sprites
            ).run_if(in_state(ProgramState::ProjectOpen))
        );

        app.add_event::<TilePlaceEvent>();
        app.add_event::<SpawnSprite>();
        app.add_event::<TileDeleteEvent>();
        app.add_event::<UpdateSpriteEvent>();
        app.add_event::<SpawnMapEntity>();
        app.add_event::<ClearTiles>();
    }
}
