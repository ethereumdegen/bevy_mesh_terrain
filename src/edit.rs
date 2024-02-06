 

use std::ops::{Div, Add, Neg};

use bevy::ecs::entity::Entity;
use bevy::math::Vec2;

use bevy::ecs::event::Event;
use bevy::prelude::EventReader;

use bevy::asset::{AssetServer, Assets};
use bevy::render::texture::Image;

use bevy::prelude::*;

use crate::chunk::{Chunk, ChunkData, ChunkHeightMapResource};
use crate::terrain::{TerrainData, TerrainImageDataLoadStatus};

#[derive(Debug)]
pub enum EditingTool {
    SetHeightMap(u16, f32), // height, radius
}

// entity, editToolType, coords, magnitude
#[derive(Event)]
pub struct EditTerrainEvent {
    pub entity: Entity,
    pub tool: EditingTool,
    pub coordinates: Vec2,
}

pub fn apply_tool_edits(
    //mut asset_server: Res<AssetServer>,
    mut chunk_query: Query<(&Chunk, &mut ChunkData, &Parent)>, //chunks parent should have terrain data
    chunk_mesh_query: Query<(&Parent, &GlobalTransform)>,
    
    mut chunk_height_maps: ResMut<ChunkHeightMapResource>,
    
    mut ev_reader: EventReader<EditTerrainEvent>,
) {
    for ev in ev_reader.read() {
        eprintln!("-- {:?} -- terrain edit event!", &ev.tool);

        let intersected_entity = &ev.entity;

        
       
        
        
      //  if let Some((chunk, mut chunk_data)) = chunk_query.get_mut(intersected_entity.clone()).ok()
        if let Some((chunk_entity, chunk_transform)) = chunk_mesh_query.get(intersected_entity.clone()).ok()
        {
                   
        if   let Some((chunk, mut chunk_data, terrain_entity )) = chunk_query.get_mut(chunk_entity.get().clone()).ok() { 
            //   if let Some(mut terrain_data) = terrain_data_query.get_mut(terrain_entity.get().clone()).ok() { //why cant i find this ?

            
             let chunk_dimensions = Vec2::new(64.0,64.0); //compute me from config 
             
            match &ev.tool {
                EditingTool::SetHeightMap(height, radius) => {
                    if let Some(height_map_data) = &mut chunk_height_maps.chunk_height_maps.get_mut(&chunk.chunk_id) {
                        // if let Some(img) = images.get_mut( height_map_image_handle ){

                        let tool_coords: &Vec2 = &ev.coordinates;
                        let chunk_transform = chunk_transform.translation();
                        let chunk_transform_vec2 : Vec2 = Vec2::new( chunk_transform.x, chunk_transform.z );
                        
                        let chunk_center_transform = chunk_transform_vec2.add( chunk_dimensions.div(2.0)  );
                        
                        let chunk_local_distance = tool_coords.distance( chunk_center_transform )  ;
                        
                        let tool_coords_local = tool_coords.add(   chunk_transform_vec2.neg()   ) ;

                        //need to make an array of all of the data indices of the terrain that will be set .. hm ?

                        let img_data_length = height_map_data.0.len();
                        //println!("trying to edit the height map via its handle :)  {}", max );

                        let mut idx_array: [usize; 2];

                        
                        let radius_clone = radius.clone();
                        //fake it for now
                        for x in 0..img_data_length {
                            for y in 0..img_data_length {   
                                let local_coords = Vec2::new(x as f32,y as f32);
                                
                                if tool_coords_local.distance(local_coords) < radius_clone {
                                     height_map_data.0[x][y] = height.clone();
                                }
                               
                            }
                        }

                        chunk_data.height_map_image_data_load_status =
                            TerrainImageDataLoadStatus::NeedsReload;
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
