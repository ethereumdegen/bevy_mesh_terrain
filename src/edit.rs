
use bevy::math::Vec2;
use bevy::ecs::entity::Entity;

use bevy::ecs::event::Event;
use bevy::prelude::EventReader;

use bevy::asset::{AssetServer,Assets};
use bevy::render::texture::Image;

use bevy::prelude::*;

use crate::chunk::{Chunk, ChunkData};
use crate::terrain::{TerrainData, TerrainImageDataLoadStatus};



#[derive(Debug)]
pub enum EditingTool {

    SetHeightMap(u16,f32) // height, radius

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
    
   mut chunk_query: Query<(&Chunk, &mut ChunkData )>, //chunks parent should have terrain data 
 //   mut terrain_data_query : Query<&mut TerrainData>,

   // mut assets: ResMut<AssetServer>,
  //  mut images: ResMut<Assets<Image>>, 

    mut ev_reader: EventReader<EditTerrainEvent>,
) {
    for ev in ev_reader.read() {
        eprintln!("-- {:?} -- terrain edit event!", &ev.tool);

 
                let intersected_entity = &ev.entity;      
              
                if let Some((chunk, mut chunk_data )) = chunk_query.get_mut(intersected_entity.clone()).ok() { //why cant i find this ? 
                    
             //   if let Some(mut terrain_data) = terrain_data_query.get_mut(terrain_entity.get().clone()).ok() { //why cant i find this ? 
                     
                     
                    match &ev.tool {
                        EditingTool::SetHeightMap(height,radius) => {
                            
                                
                                if let Some(   height_map_data )  = &mut  chunk_data.height_map_data{
                                        
                                        
                                   // if let Some(img) = images.get_mut( height_map_image_handle ){
                                        
                                        let tool_coords: &Vec2 = &ev.coordinates ;
                                        
                                        //need to make an array of all of the data indices of the terrain that will be set .. hm ? 
                                        
                                                                            
                                        let img_data_length = height_map_data.len();
                                        //println!("trying to edit the height map via its handle :)  {}", max );
                                        
                                        let mut idx_array: [usize;2]  ;
                                        
                                        
                                        //fake it for now 
                                        for x in 0 .. img_data_length {
                                              for y in 0 .. img_data_length {
                                           //      idx_array[x][y] = height;
                                            
                                                 height_map_data[x][y] = height.clone() ;
                                                 
                                            }
                                        }
                                             
                                        
                                                                        
                                       
                                        chunk_data.height_map_image_data_load_status = TerrainImageDataLoadStatus::NeedsReload;
                                        
                                  //  }
                                }
                            
                            }
             //        }

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