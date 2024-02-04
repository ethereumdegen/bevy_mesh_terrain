
use bevy::math::Vec2;
use bevy::ecs::entity::Entity;

use bevy::ecs::event::Event;
use bevy::prelude::EventReader;

#[derive(Debug)]
pub enum EditingTool {

    SetHeightMap(u32,f32) // height, radius

}

// entity, editToolType, coords, magnitude 
#[derive(Event)]
pub struct EditTerrainEvent {
    pub entity: Entity, 
    pub tool: EditingTool, 
    pub coordinates: Vec2
}




pub fn debug_tool_edits(
    mut ev_reader: EventReader<EditTerrainEvent>,
) {
    for ev in ev_reader.read() {
        eprintln!("-- {:?} -- terrain edit event!", ev.tool);
    }
}