use bevy::prelude::Component;

#[derive(Component, Debug)]
pub struct Tab {
    pub index: u32,
}

#[derive(Component)]
pub struct TabContainerMarker;

#[derive(Component)]
pub struct AddTabButtonMarker;

