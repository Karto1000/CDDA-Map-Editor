use bevy::app::{App, Plugin, Update};
use bevy::prelude::{Component, Event, Resource};
use bevy::prelude::IntoSystemConfigs;
use bevy::reflect::TypeData;
use bevy::utils::tracing::Instrument;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::Visitor;

use crate::common::io::Load;
use crate::graphics::GetTexture;
use crate::map::events::{ClearTiles, SpawnMapEntity, TileDeleteEvent, TilePlaceEvent, UpdateSpriteEvent};
use crate::map::systems::{clear_tiles_reader, spawn_map_entity_reader, SpawnSprite, update_animated_sprites};

pub(crate) mod systems;
pub(crate) mod resources;
pub(crate) mod events;
pub(crate) mod loader;


pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, spawn_map_entity_reader);
        app.add_systems(Update, clear_tiles_reader);
        app.add_systems(Update, update_animated_sprites);

        app.add_event::<TilePlaceEvent>();
        app.add_event::<SpawnSprite>();
        app.add_event::<TileDeleteEvent>();
        app.add_event::<UpdateSpriteEvent>();
        app.add_event::<SpawnMapEntity>();
        app.add_event::<ClearTiles>();
    }
}

