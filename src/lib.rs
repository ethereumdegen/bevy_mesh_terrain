
use bevy::{   prelude::*};
use chunk::{update_terrain_chunks, build_active_terrain_chunks};
use terrain::{
    load_height_map_data_from_image,
    load_terrain_texture_from_image
    };
use terrain_material::TerrainMaterial;
     
     
     
pub mod terrain;
pub mod chunk;
pub mod heightmap;
pub mod pre_mesh;
pub mod collider; 
pub mod terrain_material;
     
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
        
        app.add_plugins( MaterialPlugin::<TerrainMaterial>::default() );
        
        
        app.add_systems(Update, update_terrain_chunks);
        app.add_systems(Update, build_active_terrain_chunks.after( update_terrain_chunks ));
        
        app.add_systems(Update, load_height_map_data_from_image  ) ;
        app.add_systems(Update, load_terrain_texture_from_image  ) ;

         
    }
}