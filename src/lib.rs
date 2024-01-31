use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;

use std::time::Duration;

use chunk::{activate_terrain_chunks, destroy_terrain_chunks, despawn_terrain_chunks, build_active_terrain_chunks, finish_chunk_build_tasks, ChunkEvent};
use collision::spawn_chunk_collision_data;
use terrain::{load_height_map_data_from_image, load_terrain_texture_from_image, TerrainData};
use terrain_material::TerrainMaterial;
 
     
     
pub mod terrain;
pub mod chunk;
pub mod heightmap;
pub mod pre_mesh;
pub mod collision; 
pub mod terrain_material;



pub struct TerrainMeshPlugin {
    task_update_rate: Duration
    
}

impl Default for TerrainMeshPlugin {
    fn default() -> Self {
        Self {
           task_update_rate: Duration::from_millis(250)
        }
    }
}

impl Plugin for TerrainMeshPlugin {
    fn build(&self, app: &mut App) {
        
        app.add_plugins( MaterialPlugin::<TerrainMaterial>::default() );
        
        app.add_event::<ChunkEvent>();
        
        app.add_systems(Update, activate_terrain_chunks .run_if(on_timer(self.task_update_rate) )   );
        app.add_systems(Update, destroy_terrain_chunks .run_if(on_timer(self.task_update_rate) )   );
        app.add_systems( Last , despawn_terrain_chunks ); 
        
        app.add_systems(Update, finish_chunk_build_tasks.run_if(on_timer(self.task_update_rate) )   );
        
        app.add_systems(Update, build_active_terrain_chunks/*.after( update_terrain_chunks )*/);
        
        app.add_systems( Update, spawn_chunk_collision_data .run_if(on_timer(self.task_update_rate) )   );
        
        app.add_systems(Update, load_height_map_data_from_image  ) ;
        app.add_systems(Update, load_terrain_texture_from_image  ) ;

         
    }
    
}

impl TerrainMeshPlugin {
    
    
    pub fn get_height_texture_data( 
         terrain_data_query: Query< &TerrainData >,
         
         entity_id: Entity,
         chunk_index: u32 
         
        ) -> Vec<Vec<u16>> {
        
          if let Ok(terrain_data)  = terrain_data_query.get(entity_id) {
              
              println!("returning height texture data...");
              //terrain_data.
          }
          
          
          
        vec![]
    }
    
    pub fn set_height_texture_data( &self, chunk_index: u32, data: Vec<Vec<u16>>  ){
        
        
    }
    
}