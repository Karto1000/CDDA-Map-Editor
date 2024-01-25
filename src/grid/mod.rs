pub(crate) mod systems;

use bevy::asset::{Asset, Assets};
use bevy::math::Vec2;
use bevy::prelude::{Component, EventReader, Query, Res, ResMut, Resource, Transform, TypePath, With, Without};
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::sprite::Material2d;
use bevy::window::WindowResized;

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct GridMaterial {
    #[uniform(0)]
    pub viewport_width: f32,
    #[uniform(1)]
    pub viewport_height: f32,
    #[uniform(2)]
    pub tile_size: f32,
    #[uniform(3)]
    pub offset: Vec2,
    #[uniform(4)]
    pub mouse_pos: Vec2,
}

#[derive(Resource)]
pub struct Grid {
    pub tile_size: f32,
    pub default_tile_size: f32,
    pub offset: Vec2,

    pub min_zoom: f32,
    pub max_zoom: f32,
}

#[derive(Component)]
pub struct GridMarker;


impl Material2d for GridMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/grid.wgsl".into()
    }
}