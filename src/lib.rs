
use bevy::{   prelude::*};
use chunk::{update_terrain_chunks, build_active_terrain_chunks};
use terrain::update_terrain_data;
     
     
     
pub mod terrain;
pub mod chunk;
pub mod heightmap;
pub mod pre_mesh;
     
pub struct TerrainMeshPlugin {
    
    
}

impl Default for TerrainMeshPlugin {
    fn default() -> Self {
        Self {
           
        }
    }
}

impl Plugin for TerrainMeshPlugin {
    fn build(&self, app: &mut App) {
        
        
        app.add_systems(Update, update_terrain_chunks);
        app.add_systems(Update, build_active_terrain_chunks.after( update_terrain_chunks ));
        
        app.add_systems(Update, update_terrain_data  ) ;
        

         
    }
}