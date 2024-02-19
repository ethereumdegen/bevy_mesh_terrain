use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use chunk::{
    build_chunk_height_data, build_chunk_meshes, finish_chunk_build_tasks, initialize_chunk_data,
    reset_chunk_height_data, update_chunk_visibility, ChunkHeightMapResource,
};
use terrain::{initialize_terrain, load_terrain_texture_from_image};
 


use std::time::Duration;
 

use terrain_material::TerrainMaterial;
use crate::chunk::TerrainMaterialExtension;

use edit::{apply_tool_edits, apply_command_events, finish_chunk_collision_save_tasks, EditTerrainEvent, TerrainCommandEvent};

pub mod chunk;
//pub mod collision;
pub mod edit;
pub mod heightmap;
pub mod pre_mesh;
pub mod terrain;
pub mod terrain_config;
pub mod terrain_material;
pub mod terrain_loading_state;



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

       

        app.add_plugins(MaterialPlugin::<TerrainMaterialExtension>::default());
        
        app.add_state::<terrain_loading_state::TerrainLoadingState>();

        //app.add_event::<ChunkEvent>();
        app.add_event::<EditTerrainEvent>();
        app.add_event::<TerrainCommandEvent>();
        app.insert_resource(ChunkHeightMapResource::default());

        app.add_systems(
            Update,
            chunk::update_splat_image_formats
        );

        app.add_systems(
            Update,
            initialize_chunk_data.run_if(on_timer(self.task_update_rate)),
        );

        app.add_systems(
            Update,
            reset_chunk_height_data.run_if(on_timer(self.task_update_rate)),
        );
        app.add_systems(
            Update,
            build_chunk_height_data.run_if(on_timer(self.task_update_rate)),
        );
        app.add_systems(
            Update,
            finish_chunk_build_tasks.run_if(on_timer(self.task_update_rate)),
        ); 
        app.add_systems(
            Update,
            initialize_terrain.run_if(on_timer(self.task_update_rate)),
        );
        app.add_systems(
            Update,
            build_chunk_meshes.run_if(on_timer(self.task_update_rate)),
        );
        app.add_systems(
            Update,
            update_chunk_visibility.run_if(on_timer(self.task_update_rate)),
        );

        app.add_systems(Update, load_terrain_texture_from_image);
 
        app.add_systems(Update, (apply_tool_edits,apply_command_events ));
        
        app.add_systems(
            Update,
            finish_chunk_collision_save_tasks.run_if(on_timer(self.task_update_rate)),
        ); 

        
    }
}
