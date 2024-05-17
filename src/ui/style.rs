use bevy::prelude::{Color, Resource};

#[derive(Debug, Resource)]
pub struct Style {
    pub gray_dark: Color,
    pub gray_darker: Color,
    pub gray_light: Color,
    pub white: Color,
    pub black: Color,
    pub blue_light: Color,
    pub blue_dark: Color,
    pub blue_darkest: Color,
    pub selected: Color,
    pub error: Color,
}

impl Style {
    pub fn dark() -> Self {
        return Self {
            gray_dark: Color::rgb(0.11, 0.11, 0.11),
            gray_darker: Color::rgb(0.05, 0.05, 0.05),
            gray_light: Color::rgb(0.15, 0.15, 0.15),
            selected: Color::rgb(0.23, 0.54, 0.95),
            white: Color::rgb(1., 1., 1.),
            black: Color::rgb(0., 0., 0.),
            blue_light: Color::rgb(0.525, 0.760, 0.956),
            blue_dark: Color::rgb(0.137, 0.254, 0.431),
            blue_darkest: Color::rgb(0.117, 0.160, 0.264),
            error: Color::rgb(0.97, 0.41, 0.41),
        };
    }

    pub fn light() -> Self {
        todo!()
    }
}