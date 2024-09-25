//use crate::terrain::TerrainSceneDataResource;
use bevy::time::common_conditions::on_timer;
use bevy::{asset::load_internal_asset, prelude::*};
use chunk::{
     build_chunk_meshes, finish_chunk_build_tasks, initialize_chunk_data,
     update_chunk_visibility,
};
use terrain::{initialize_terrain, load_terrain_texture_from_image, load_terrain_normal_from_image};

use std::time::Duration;

//use chunk::{activate_terrain_chunks, destroy_terrain_chunks, despawn_terrain_chunks, build_active_terrain_chunks, finish_chunk_build_tasks, ChunkEvent};
//use collision::spawn_chunk_collision_data;

use crate::chunk::TerrainMaterialExtension;
use crate::terrain_material::TERRAIN_SHADER_HANDLE;
use terrain_material::TerrainMaterial;


/*
use edit::{
    apply_command_events, apply_tool_edits, EditTerrainEvent, TerrainBrushEvent,
    TerrainCommandEvent,
};
*/

pub mod chunk;
//pub mod collision;
//pub mod edit;
pub mod heightmap;
pub mod pre_mesh;
pub mod terrain;
pub mod terrain_config;
pub mod terrain_loading_state;
pub mod terrain_material;
pub mod tool_preview;
pub mod terrain_scene;

pub struct TerrainMeshPlugin {
    task_update_rate: Duration,
}

impl Default for TerrainMeshPlugin {
    fn default() -> Self {
        Self {
            task_update_rate: Duration::from_millis(250),
        }
    }
}
impl Plugin for TerrainMeshPlugin {
    fn build(&self, app: &mut App) {
        // load terrain shader into cache
        load_internal_asset!(
            app,
            TERRAIN_SHADER_HANDLE,
            "shaders/terrain.wgsl",
            Shader::from_wgsl
        );
        app.add_plugins(MaterialPlugin::<TerrainMaterialExtension>::default());

    //    app.add_plugins(edit::terrain_edit_plugin) ;
        
        app.init_state::<terrain_loading_state::TerrainLoadingState>();

        app.init_resource::<tool_preview::ToolPreviewResource>();



        //app.add_event::<ChunkEvent>();
       
       // app.insert_resource(TerrainSceneDataResource::default());

        //app.add_systems(Update, chunk::update_splat_image_formats);
        app.add_systems(Update, chunk::update_tool_uniforms);

        app.add_systems(
            Update,
            initialize_chunk_data.run_if(on_timer(self.task_update_rate)),
        );

       /* app.add_systems(
            Update,
            reset_chunk_height_data.run_if(on_timer(self.task_update_rate)),
        );*/
        /*app.add_systems(
            Update,
            build_chunk_height_data.run_if(on_timer(self.task_update_rate)),
        );*/
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

        //this is for the diffuse and normal 
        app.add_systems(Update, (load_terrain_texture_from_image,load_terrain_normal_from_image));
 
    }
}
