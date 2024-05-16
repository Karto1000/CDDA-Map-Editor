use bevy::math::Vec2;
use bevy::prelude::{BackgroundColor, Button, Changed, Entity, Event, EventReader, EventWriter, GlobalTransform, Query, ResMut, Vec3Swizzles, Visibility, Window, With};
use bevy::ui::{Interaction, Node};
use bevy::window::PrimaryWindow;

use crate::IsCursorCaptured;
use crate::ui::components::{HoverEffect, ToggleEffect};

pub fn button_hover_system(
    mut q_interaction: Query<(
        Entity,
        &Interaction,
        &mut BackgroundColor,
        &HoverEffect,
    ), (
        Changed<Interaction>,
        With<Button>
    )>,
    q_toggle: Query<&ToggleEffect>,
) {
    for (entity, interaction, mut background_color, hover_effect) in q_interaction.iter_mut() {
        match *interaction {
            Interaction::Pressed => {}
            Interaction::Hovered => {
                match q_toggle.get(entity) {
                    Ok(e) => {
                        if e.toggled { return; }
                    }
                    Err(_) => {}
                };

                background_color.0 = hover_effect.hover_color;
            }
            Interaction::None => {
                match q_toggle.get(entity) {
                    Ok(e) => {
                        if e.toggled { return; }
                    }
                    Err(_) => {}
                };

                background_color.0 = hover_effect.original_color;
            }
        }
    }
}

#[derive(Event)]
pub struct ResetToggle {
    ignore: Entity,
}

pub fn reset_toggle_reader(
    mut q_buttons: Query<(Entity, &mut BackgroundColor, &mut ToggleEffect), With<Button>>,
    mut e_reset_toggle: EventReader<ResetToggle>,
) {
    for event in e_reset_toggle.read() {
        for (entity, mut background_color, mut toggle_effect) in q_buttons.iter_mut() {
            if entity == event.ignore { continue; }

            background_color.0 = toggle_effect.original_color;
            toggle_effect.toggled = false;
        }
    }
}

pub fn button_toggle_system(
    mut q_interaction: Query<(Entity, &Interaction, &mut BackgroundColor, &mut ToggleEffect), (Changed<Interaction>, With<Button>)>,
    mut e_reset_toggle_writer: EventWriter<ResetToggle>,
) {
    for (entity, interaction, mut background_color, mut toggle_effect) in q_interaction.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                match toggle_effect.toggled {
                    true => background_color.0 = toggle_effect.original_color,
                    false => background_color.0 = toggle_effect.toggled_color
                }
                toggle_effect.toggled = !toggle_effect.toggled;
                e_reset_toggle_writer.send(ResetToggle { ignore: entity });
            }
            Interaction::Hovered => {}
            Interaction::None => {}
        }
    }
}

pub fn check_ui_interaction(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut res_cursor: ResMut<IsCursorCaptured>,
    node_query: Query<(&Node, &GlobalTransform, &Visibility)>,
) {
    let cursor_position = windows.single().cursor_position().unwrap_or(Vec2::default());
    res_cursor.0 = node_query.iter()
        .any(|(&node, &transform, &visibility)| {
            let node_position = transform.translation().xy();
            let half_size = 0.5 * node.size();
            let min = node_position - half_size;
            let max = node_position + half_size;
            (min.x..max.x).contains(&cursor_position.x)
                && (min.y..max.y).contains(&cursor_position.y)
        });
}