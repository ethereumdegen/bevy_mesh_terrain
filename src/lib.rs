
use std::time::Duration;

use bevy::{   prelude::*, time::common_conditions::on_timer};
use chunk::{activate_terrain_chunks, destroy_terrain_chunks, build_active_terrain_chunks, finish_chunk_build_tasks, ChunkEvent};
use collision::spawn_chunk_collision_data;
use terrain::{
    load_height_map_data_from_image,
    load_terrain_texture_from_image
    };
use terrain_material::TerrainMaterial;
 
     
     
pub mod terrain;
pub mod chunk;
pub mod heightmap;
pub mod pre_mesh;
pub mod collision; 
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
        
        app.add_event::<ChunkEvent>();
        
        app.add_systems(Update, activate_terrain_chunks .run_if(on_timer(Duration::from_millis(100)) )   );
        app.add_systems(Update, destroy_terrain_chunks .run_if(on_timer(Duration::from_millis(100)) )   );
        
        app.add_systems(Update, finish_chunk_build_tasks.run_if(on_timer(Duration::from_millis(100)) )   );
        
        app.add_systems(Update, build_active_terrain_chunks/*.after( update_terrain_chunks )*/);
        
        app.add_systems( Update, spawn_chunk_collision_data .run_if(on_timer(Duration::from_millis(100)) )   );
        
        app.add_systems(Update, load_height_map_data_from_image  ) ;
        app.add_systems(Update, load_terrain_texture_from_image  ) ;

         
    }
}