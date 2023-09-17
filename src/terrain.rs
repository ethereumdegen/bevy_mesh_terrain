use bevy::{   prelude::*, utils::HashMap, render::render_resource::Texture, asset::LoadState};

use crate::{chunk::ChunkData, heightmap::HeightMapU16};
     
 use crate::heightmap::HeightMap;

//attach me to camera 
#[derive(Component,Default)]
pub struct TerrainViewer {
    
}


#[derive(Component)]
pub struct TerrainConfig {
    
    pub terrain_dimensions: Vec2,  
    
  //  pub chunk_width: f32,
    pub chunk_rows: u32,
    
    pub render_distance: f32, 
}

impl Default for TerrainConfig {
    fn default() -> Self {
        Self {
           // chunk_width: 64.0 ,
            terrain_dimensions: Vec2::new(1024.0,1024.0),
            chunk_rows: 64 ,
            render_distance: 400.0, 
        }
    }
}

impl TerrainConfig {
    
     pub fn get_chunk_dimensions(&self ) -> Vec2 {
        let chunk_dimension_x = self.terrain_dimensions.x / self.chunk_rows as f32;
        let chunk_dimension_z = self.terrain_dimensions.y / self.chunk_rows as f32;
         
        
        Vec2::new(chunk_dimension_x, chunk_dimension_z) 
        
    }  
    
    pub fn get_chunk_render_distance(&self) -> u32{
        return self.render_distance as u32 / self.chunk_rows; 
    }
    
}

#[derive(Component,Default)]
pub struct TerrainData {
      //  pub terrain_origin: Vec3 // should be a component of an entity 
    //chunk_index -> chunk data 
    //chunk index is   chunk_col * 64  + chunk_row   IF chunk_rows is 64 
    //this only tracks loaded and active chunks and these are all entities 
    pub chunks: HashMap<u32,ChunkData>, 
    
    //could be a massive image like 4k 
    height_map_image_handle: Option<Handle<Image>>, 
    //need to add asset handles here for the heightmap image and texture image !!! 
    
     
    pub height_map_data: Option<HeightMapU16> 
}
 
impl TerrainData{
    
    pub fn add_height_map_image(&mut self, handle: Handle<Image> ){
        self.height_map_image_handle = Some(handle.clone()); //strong clone 
    }
    
}
 
 //finalizes loading of height map by looking for image handle and applying it to the height map data 
pub fn update_terrain_data(  
    
    mut terrain_query: Query<(Entity, &TerrainConfig,&mut TerrainData)>,
    
    //terrain_viewer: Query<&Transform, With<TerrainViewer>>
    asset_server: Res<AssetServer>,
    
    //assets -- temp 
    images: Res<Assets<Image>>,
  
    
){
    
    
    for (terrain_entity, terrain_config, mut terrain_data) in terrain_query.iter_mut() { 
        
        
        let height_map_data_is_some = terrain_data.height_map_data.is_some(); 
         
         if !height_map_data_is_some {   
                
                //try to get the loaded height map image from its handle via the asset server - must exist and be loaded 
                let height_map_image:&Image = match &terrain_data.height_map_image_handle {
                    Some(height_map_handle) => {
                        
                        let height_map_loaded = asset_server.get_load_state( height_map_handle )  ;
                    
                        if height_map_loaded != LoadState::Loaded  {
                            println!("height map not yet loaded");
                            continue;
                        }  
                        
                        images.get(height_map_handle).unwrap()
                    }
                    None => {continue} 
                };
                
                let loaded_heightmap_data_result =  HeightMapU16::load_from_image( height_map_image) ;
                   
                match loaded_heightmap_data_result {
                    Ok( loaded_heightmap_data ) => {
                         println!("loaded height map data from image");
                       //take out of box 
                            terrain_data.height_map_data = Some( *loaded_heightmap_data ); 
                 
                    },
                    Err(e) => {
                        
                        println!("{}",e);
                    }
                    
                }
                   
            
               
                //we can let go of the height map image handle now that we loaded our heightmap data from it 
                terrain_data.height_map_image_handle = None;
         }
         
        
        
    }
}
 