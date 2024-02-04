
use bevy::math::Vec2;
use bevy::ecs::entity::Entity;

use bevy::ecs::event::Event;
use bevy::prelude::EventReader;

use bevy::asset::AssetServer;


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



pub fn apply_tool_edits(
    mut asset_server: AssetServer,

    mut ev_reader: EventReader<EditTerrainEvent>,
) {
    for ev in ev_reader.read() {
        eprintln!("-- {:?} -- terrain edit event!", &ev.tool);


        match &ev.tool {
            EditingTool::SetHeightMap(height,radius) => {
                // this should edit the height map  as it is inside of Asset<> memory... i think .. 


            }
        }


    }
}

/*
pub fn debug_tool_edits(
    mut ev_reader: EventReader<EditTerrainEvent>,
) {
    for ev in ev_reader.read() {
        eprintln!("-- {:?} -- terrain edit event!", ev.tool);
    }
}
*/