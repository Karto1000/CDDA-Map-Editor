use bevy::math::Vec2;
use bevy::prelude::Resource;

#[derive(Resource, Debug)]
pub struct PlaceInfo {
    pub last_place_position: Option<Vec2>,
}