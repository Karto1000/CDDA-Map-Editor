use bevy::app::{App, Plugin, Update};
use bevy::asset::Handle;
use bevy::prelude::{Commands, Component, default, Entity, Event, EventReader, EventWriter, Image, Query, Res, ResMut, Resource, SpriteBundle, Transform, Vec3, With};
use bevy::prelude::IntoSystemConfigs;
use bevy::reflect::TypeData;
use bevy::utils::tracing::Instrument;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::Visitor;

use crate::{EditorData, GraphicsResource};
use crate::graphics::{GetTexture, LegacyTextures};
use crate::grid::resources::Grid;
use crate::map::events::{ClearTiles, SpawnMapEntity, TileDeleteEvent, TilePlaceEvent, UpdateSpriteEvent};
use crate::map::systems::{clear_tiles_reader, map_save_system, save_directory_picked, spawn_map_entity_reader, update_sprite_reader};
use crate::tiles::components::Tile;

pub(crate) mod systems;
pub(crate) mod resources;
pub(crate) mod events;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, map_save_system);
        app.add_systems(Update, save_directory_picked);
        app.add_systems(Update, spawn_map_entity_reader);
        app.add_systems(Update, clear_tiles_reader);
        app.add_systems(Update, update_sprite_reader);

        app.add_event::<TilePlaceEvent>();
        app.add_event::<TileDeleteEvent>();
        app.add_event::<UpdateSpriteEvent>();
        app.add_event::<SpawnMapEntity>();
        app.add_event::<ClearTiles>();
    }
}

