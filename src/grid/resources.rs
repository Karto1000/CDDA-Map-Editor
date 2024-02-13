use bevy::math::Vec2;
use bevy::prelude::Resource;

#[derive(Resource, Debug)]
pub struct DragInfo {
    pub drag_started: Option<Vec2>,
    pub last_position: Option<Vec2>,
}

#[derive(Resource)]
pub struct Grid {
    pub tile_size: f32,
    pub default_tile_size: f32,
    pub offset: Vec2,

    pub min_zoom: f32,
    pub max_zoom: f32,
}