use bevy::app::{App, Plugin, Update};
use bevy::asset::Asset;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::{Component, in_state, IntoSystemConfigs, TypePath};
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::sprite::Material2d;

use crate::program::data::ProgramState;
use crate::ui::grid::resources::{DragInfo, Grid};
use crate::ui::grid::systems::{
    drag_system, grid_resize_system, window_grid_resize_system,
};

pub(crate) mod systems;
pub(crate) mod resources;

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        let grid: Grid = Grid {
            tile_size: 32.0,
            default_tile_size: 32.0,
            offset: Vec2::new(0., 0.),
            min_zoom: 6.,
            max_zoom: 128.,
            instantiated_grid: None,
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
                drag_system,
            ).run_if(in_state(ProgramState::ProjectOpen)),
        );

        app.insert_resource(drag_info);
        app.insert_resource(grid);
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct GridMaterial {
    #[uniform(0)]
    pub tile_size: f32,
    #[uniform(1)]
    pub offset: Vec2,
    #[uniform(2)]
    pub mouse_pos: Vec2,
    #[uniform(3)]
    pub map_size: Vec2,
    #[uniform(4)]
    // This is an i32 because bevy won't let me pass a bool as a uniform
    pub is_cursor_captured: i32,
    #[uniform(5)]
    pub scale_factor: f32,

    #[uniform(6)]
    pub inside_grid_color: Vec3,
    #[uniform(7)]
    pub outside_grid_color: Vec3,
}


#[derive(Component)]
pub struct GridMarker;

impl Material2d for GridMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/grid.wgsl".into()
    }
}
