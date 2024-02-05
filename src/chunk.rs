use bevy::prelude::*;
use bevy::tasks::{Task, AsyncComputeTaskPool};

use bevy::utils::HashMap;
use futures_lite::future;

use crate::heightmap::{SubHeightMapU16, HeightMapU16};
use crate::pre_mesh::PreMesh;
use crate::terrain::{ TerrainViewer, TerrainData, TerrainImageDataLoadStatus};
use crate::terrain_material::{TerrainMaterial, ChunkMaterialUniforms};



#[derive(Default,Eq,PartialEq)]
enum ChunkState{
    #[default]
    Init,
    
    Building, 
    FullyBuilt,
}


#[derive(Event )]
pub enum ChunkEvent {
    ChunkEntitySpawned(Entity)
} 


#[derive(Component,Default)]
pub struct Chunk {
    pub chunk_id: u32, //same as chunk index   
  //  pub chunk_bounds: [[usize;2]; 2 ],
   // pub chunk_state: ChunkState,
   // pub lod_level: Option<u8>
    
} 

impl Chunk {
    
    pub fn new (chunk_id:u32  ) -> Self {
        
        Self {
            chunk_id,
           // chunk_bounds,
          //  chunk_state: ChunkState::Init,
            
          //  lod_level: 
            
        }
        
        
    }
}


#[derive(Component)]
pub struct ChunkData {
    spawned_mesh_entity: Option<Entity> ,
    chunk_state: ChunkState ,
    lod_level: u8, 
  
    
    //could be a massive image like 4k 
    pub height_map_image_handle: Option<Handle<Image>>, 
    pub height_map_image_data_load_status: TerrainImageDataLoadStatus,
    
    //need to add asset handles here for the heightmap image and texture image !!! 
     
     
    pub height_map_data: Option<HeightMapU16>,
   
    
 //   texture_image_handle: Option<Handle<Image>>,
 //   texture_image_sections: u32, 
 //   texture_image_finalized: bool,  //need this for now bc of the weird way we have to load an array texture w polling and stuff... GET RID of me ???replace w enum ? 
    
    splat_image_handle: Option<Handle<Image>>,
    
    alpha_mask_image_handle: Option<Handle<Image>>, //built from the height map 
   
    pub terrain_material_handle: Option<Handle<TerrainMaterial> >
}


   
 
 


pub trait ChunkCoordinates {
    
    fn x(&self) -> u32 ;
    fn y(&self) -> u32 ;
    
    fn get_chunk_index(&self, chunk_rows: u32) -> u32; 


    fn from_location( location: Vec3 ,  terrain_origin: Vec3 , terrain_dimensions: Vec2 , chunk_rows: u32 ) -> Option<UVec2> ;
    fn to_location(&self, terrain_origin: Vec3, terrain_dimensions: Vec2, chunk_rows: u32) -> Option<Vec3> ;
    
    fn from_chunk_id(chunk_id:u32,chunk_rows:u32) -> Self;
    fn get_location_offset(&self,  chunk_dimensions: Vec2 ) -> Vec3; 
    
    fn get_heightmap_subsection_bounds_pct(&self, chunk_rows:u32 ) -> [ [f32 ; 2]  ;2 ] ; 
}


type ChunkCoords =  [u32; 2 ] ; 

impl ChunkCoordinates for  ChunkCoords {
    
     fn x(&self) -> u32 {
        self[0]
    }
     fn y(&self) -> u32 {
        self[1]
    }
    
     //chunk index is   chunk_col * 64  + chunk_row   IF chunk_rows is 64 
    fn get_chunk_index(&self, chunk_rows: u32) -> u32 {
        
        return self.y() * chunk_rows + self.x() as u32; 
        
    }
    
    
    fn from_chunk_id(chunk_id:u32, chunk_rows: u32) -> Self { 
        let coords_y = chunk_id / chunk_rows;
        let coords_x = chunk_id % chunk_rows;
        
        [coords_x,coords_y]
    }
      
      
    
    
    fn get_location_offset(&self,  chunk_dimensions: Vec2 ) -> Vec3 { 
         
        Vec3::new(chunk_dimensions.x * self.x() as f32,0.0,chunk_dimensions.y * self.y() as f32) 
        
    }  
        
        
    fn from_location(from_location: Vec3, terrain_origin: Vec3, terrain_dimensions: Vec2, chunk_rows: u32) -> Option<UVec2> {
        let location_delta = from_location - terrain_origin;

        //let terrain_min = terrain_origin;
        //let terrain_max = terrain_origin + Vec3::new(terrain_dimensions.x, 0.0, terrain_dimensions.y);

        // Check if from_location is within the terrain bounds
        if location_delta.x >= 0.0 && location_delta.x <= terrain_dimensions.x && 
           location_delta.z >= 0.0 && location_delta.z <= terrain_dimensions.y {

            // Calculate the chunk's x and z coordinates
            let chunk_x = (location_delta.x / terrain_dimensions.x * chunk_rows as f32) as u32;
            let chunk_z = (location_delta.z / terrain_dimensions.y * chunk_rows as f32) as u32;

            return Some(UVec2::new(chunk_x, chunk_z));
        }

        None
    }
    
    //returns the middle of the chunk 
    fn to_location(&self, terrain_origin: Vec3, terrain_dimensions: Vec2, chunk_rows: u32) -> Option<Vec3> {
    // Ensure chunk coordinates are within bounds
    if self.x() < chunk_rows && self.y() < chunk_rows {
        // Calculate the dimensions of a single chunk
        let chunk_dim_x = terrain_dimensions.x / chunk_rows as f32;
        let chunk_dim_z = terrain_dimensions.y / chunk_rows as f32;

        // Calculate world location for the bottom-left corner of the chunk
        let world_x = terrain_origin.x + self.x() as f32 * chunk_dim_x + chunk_dim_x/2.0;
        let world_z = terrain_origin.z + self.y() as f32 * chunk_dim_z + chunk_dim_z/2.0;
        
        

        return Some(Vec3::new(world_x, terrain_origin.y, world_z));
    }

    None
    }
    
     fn get_heightmap_subsection_bounds_pct(
         &self,
         chunk_rows: u32
         
         ) -> [ [f32 ; 2]  ;2 ] {
        let chunk_x = self.x();
        let chunk_y = self.y();
        
        let pct_per_row = 1.0 / chunk_rows as f32;  
        
        return [
            [ chunk_x as f32 * pct_per_row , chunk_y as f32 * pct_per_row ],  //start corner x and y 
            [(chunk_x+1) as f32 * pct_per_row , (chunk_y+1) as f32 * pct_per_row]    //end corner x and y 
        ]
    }
    
    
}

  
  
  
fn calculate_chunk_coords( from_location: Vec3, terrain_origin: Vec3, terrain_dimensions: Vec2, chunk_rows: u32  ) -> [ i32 ;2] {
     
        let location_delta = from_location - terrain_origin;

        
        let chunk_x = (location_delta.x / terrain_dimensions.x * chunk_rows as f32) as i32;
        let chunk_z = (location_delta.z / terrain_dimensions.y * chunk_rows as f32) as i32;

        return  [chunk_x, chunk_z] ; 
    
}
  
  
  
/*

On initialization of terrain entity, the chunk entities should be spawned and they should just remain there forever !!! 
 */ 