use bevy::prelude::Event;

#[derive(Event)]
pub struct SpawnTab {
    pub name: String,
    pub index: u32,
}