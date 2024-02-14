use std::sync::Arc;

use bevy::prelude::Event;

use crate::common::Coordinates;
use crate::map::resources::MapEntity;
use crate::map::systems::SpriteKind;
use crate::tiles::components::Tile;

#[derive(Event)]
pub struct UpdateSpriteEvent {
    pub tile: Tile,
    pub coordinates: Coordinates,
}

#[derive(Event, Debug)]
pub struct TilePlaceEvent {
    pub tile: Tile,
    pub coordinates: Coordinates,
    pub should_update_sprites: bool,
}

#[derive(Event, Debug)]
pub struct TileDeleteEvent {
    pub tile: Tile,
    pub coordinates: Coordinates,
}

#[derive(Event)]
pub struct SpawnMapEntity {
    pub map_entity: Arc<MapEntity>,
}

#[derive(Event)]
pub struct ClearTiles;