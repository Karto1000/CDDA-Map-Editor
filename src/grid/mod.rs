use bevy::app::{App, Plugin, Update};
use bevy::asset::Asset;
use bevy::math::Vec2;
use bevy::prelude::{Component, Resource, TypePath};
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::sprite::Material2d;

use crate::grid::resources::{DragInfo, Grid};
use crate::grid::systems::{
    drag_system, grid_resize_system, map_resize_system, window_grid_resize_system,
};

pub(crate) mod systems;
pub(crate) mod resources;

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        let grid: Grid = Grid {
            tile_size: 32.0,
            map_size: Vec2 { x: 24., y: 24. },
            default_tile_size: 32.0,
            offset: Vec2::new(0., 0.),
            min_zoom: 6.,
            max_zoom: 128.,
        };

        let drag_info: DragInfo = DragInfo {
            drag_started: None,
            last_position: None,
        };

        app.add_systems(
            Update,
            (
                window_grid_resize_system,
                grid_resize_system,
                map_resize_system,
                drag_system,
            ),
        );

        app.insert_resource(drag_info);
        app.insert_resource(grid);
    }
}

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
    #[uniform(5)]
    pub map_size: Vec2,
}


#[derive(Component)]
pub struct GridMarker;

impl Material2d for GridMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/grid.wgsl".into()
    }
}
