
use bevy::math::Vec2;
use bevy::ecs::entity::Entity;

use bevy::ecs::event::Event;
use bevy::prelude::EventReader;

use bevy::asset::{AssetServer,Assets};
use bevy::render::texture::Image;

use bevy::prelude::*;

use crate::chunk::Chunk;
use crate::terrain::{TerrainData, TerrainImageDataLoadStatus};



#[derive(Debug)]
pub enum EditingTool {

    SetHeightMap(u8,f32) // height, radius

}

// entity, editToolType, coords, magnitude 
#[derive(Event)]
pub struct EditTerrainEvent {
    pub entity: Entity, 
    pub tool: EditingTool, 
    pub coordinates: Vec2
}

 


pub fn apply_tool_edits(
    //mut asset_server: Res<AssetServer>,
    
    chunk_query: Query<(&Chunk, &Parent)>, //chunks parent should have terrain data 
    mut terrain_data_query : Query<&mut TerrainData>,

   // mut assets: ResMut<AssetServer>,
    mut images: ResMut<Assets<Image>>, 

    mut ev_reader: EventReader<EditTerrainEvent>,
) {
    for ev in ev_reader.read() {
        eprintln!("-- {:?} -- terrain edit event!", &ev.tool);

 
                let intersected_entity = &ev.entity;      
              
                if let Some((chunk, terrain_entity)) = chunk_query.get(intersected_entity.clone()).ok() { //why cant i find this ? 
                    
                if let Some(mut terrain_data) = terrain_data_query.get_mut(terrain_entity.get().clone()).ok() { //why cant i find this ? 
                     
                     
                    match &ev.tool {
                        EditingTool::SetHeightMap(height,radius) => {
                            
                                
                                if let Some( height_map_image_handle )  = &terrain_data.height_map_image_handle{
                                        
                                        
                                    if let Some(img) = images.get_mut( height_map_image_handle ){
                                        
                                        let tool_coords: &Vec2 = &ev.coordinates ;
                                        
                                        //need to make an array of all of the data indices of the terrain that will be set .. hm ? 
                                        
                                                                            
                                        let img_data_length = img.data.len();
                                        //println!("trying to edit the height map via its handle :)  {}", max );
                                        
                                        let mut idx_array = vec![22,23,24,25,26,27,28];
                                        
                                        for i in 0 .. img_data_length {
                                            idx_array.push(i);
                                        }
                                        
                                        for idx in idx_array {
                                             img.data[idx] = height.clone() ;
                                        
                                        }                                        
                                       
                                        terrain_data.height_map_image_data_load_status = TerrainImageDataLoadStatus::NeedsReload;
                                        
                                    }
                                }
                            
                            }
                     }

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