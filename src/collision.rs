use bevy::prelude::*;

use crate::chunk::{Chunk, ChunkCoordinates, ChunkData, ChunkHeightMapResource, TerrainChunkMesh};
use crate::heightmap::HeightMapU16;
use crate::terrain::{TerrainData, TerrainViewer};
use crate::terrain_config::TerrainConfig;
use crate::terrain_loading_state::TerrainLoadingState;
use bevy_mod_sysfail::*;
use anyhow::{Result,Context};

use bevy_xpbd_3d::prelude::Collider;

/*

use parry3d  heightfield  collider

get the data from the heightmap u16 ! to build a heightfield collider
Can spawn a collider on each entity that has the 'Chunk' component... the collider for that chunk

*/

/*
It is the end users responsibility to detect for these components   and  add  their own collider component . The component AddedCollisionData can be removed .

finish me !!! This should easily plug in to bevy xpbd
*/
/*
#[derive(Component)]
pub struct ChunkCollisionData {
    //consider only providing the bounds and not the actual data
    pub heightmap: HeightMapU16, //add heightfield

                                 //dont need translation since its on entity alrdy ...
}

#[derive(Component)]
pub struct AddedChunkCollisionData {}
*/

/*
pub fn spawn_chunk_collision_data(
    mut commands: Commands,
    chunk_query: Query<(Entity, &mut Chunk, &ChunkData), Without<AddedChunkCollisionData>>,

    mut chunk_height_maps: ResMut<ChunkHeightMapResource>,
    
) {
    for (chunk_entity, chunk, chunk_data) in chunk_query.iter() {
       
        let chunk_id = chunk.chunk_id;
       
        let height_map_data = chunk_height_maps.chunk_height_maps.get(&chunk.chunk_id); //&chunk_data.height_map_data.clone();

        if height_map_data.is_none() {
            continue;
        }
        let height_map_data_cloned = (&height_map_data.as_ref().unwrap().0).clone();
         

        if let Some(mut entity_commands) = commands.get_entity(chunk_entity) {
            entity_commands
                .insert(ChunkCollisionData {
                    heightmap: height_map_data_cloned,
                })
                .insert(AddedChunkCollisionData {});
        }

        
    }
}
*/
 
pub(crate) fn build_colliders_for_terrain(
    mut commands:Commands,
    meshes: Res <Assets<Mesh>>,

    chunk_mesh_query: Query<  (Entity,&Handle<Mesh>, &GlobalTransform), (With<TerrainChunkMesh>, Without<ColliderData>) >,

   // terrain_viewer: Query<&GlobalTransform, With<TerrainViewer>>,
    mut terrain_loading_state: ResMut<NextState<TerrainLoadingState>>,
) -> Result<()> {
 
   /* let viewer = terrain_viewer.get_single();

    let viewer_location: Vec3 = match viewer {
        Ok(view) => view.translation(),
        // FIX: probably should log a warning if there are multiple (or no) viewers, rather than just setting to the origin
        Err(_e) => Vec3::new(0.0, 0.0, 0.0),
    };
*/
      
     // terrain_loading_state.set(TerrainLoadingState::Ready);
      
     for (entity, mesh_handle,  mesh_transform) in chunk_mesh_query.iter(){
        
            
        //  let chunk_world_location = terrain_mesh_transform.translation();

        //  let distance_to_chunk: f32 = chunk_world_location.distance(viewer_location);

        //  if distance_to_chunk > 500.0 {
       //     continue
        //  }
          
           // terrain_loading_state.set(TerrainLoadingState::Loading);

          // i need to SIGNIFICANTLY speed this up !   
          //maybe the terrain collision data can be BUILT (baked) and loaded from a file... 

            println!("adding terrain collider"); 
           let mesh = meshes.get(mesh_handle).context("No mesh found for terrain chunk")?;

           //convex_hull_from_mesh was the old style 
           let collider = Collider::trimesh_from_mesh(&mesh).context("Failed to create collider from mesh")?; 
           
           commands.entity(entity).insert(collider);
           //why does this freeze ? 

           println!("added terrain collider to {:?}", entity); 
           
     }
     
     Ok(())
}

 
