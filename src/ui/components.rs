use bevy::prelude::{Color, Component};

#[derive(Component)]
pub struct OriginalColor(pub Color);


#[derive(Component, Debug)]
pub struct HoverEffect {
    pub original_color: Color,
    pub hover_color: Color,
}

#[derive(Component, Debug)]
pub struct ToggleEffect {
    pub original_color: Color,
    pub toggled_color: Color,
    pub toggled: bool,
}

