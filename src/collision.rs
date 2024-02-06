use bevy::prelude::*;

use crate::chunk::{Chunk, ChunkCoordinates, ChunkData};
use crate::heightmap::HeightMapU16;
use crate::terrain::TerrainData;
use crate::terrain_config::TerrainConfig;

/*

use parry3d  heightfield  collider

get the data from the heightmap u16 ! to build a heightfield collider
Can spawn a collider on each entity that has the 'Chunk' component... the collider for that chunk

*/

/*
It is the end users responsibility to detect for these components   and  add  their own collider component . The component AddedCollisionData can be removed .

finish me !!! This should easily plug in to bevy xpbd
*/
#[derive(Component)]
pub struct ChunkCollisionData {
    //consider only providing the bounds and not the actual data
    pub heightmap: HeightMapU16, //add heightfield

                                 //dont need translation since its on entity alrdy ...
}

#[derive(Component)]
pub struct AddedChunkCollisionData {}

pub fn spawn_chunk_collision_data(
    mut commands: Commands,
    chunk_query: Query<(Entity, &mut Chunk, &ChunkData), Without<AddedChunkCollisionData>>,
    //   mut terrain_query : Query<(&mut TerrainData,&TerrainConfig)> ,
) {
    for (chunk_entity, chunk, chunk_data) in chunk_query.iter() {
        //  if let Ok( (terrain_data, terrain_config) ) = terrain_query.get_mut( parent_terrain_entity.get() ){

        //     if !terrain_config.attach_collision_data {  continue }

        let chunk_id = chunk.chunk_id;
        // let chunk_rows = terrain_config.chunk_rows;
        //let terrain_dimensions = terrain_config.terrain_dimensions;

        //   let chunk_coords:[u32;2] = ChunkCoordinates::from_chunk_id(chunk_id.clone(), chunk_rows);
        //let chunk_dimensions = terrain_config.get_chunk_dimensions(  );

        //   let height_map_subsection_pct = chunk_coords.get_heightmap_subsection_bounds_pct(chunk_rows);

        let height_map_data = &chunk_data.height_map_data.clone();

        if height_map_data.is_none() {
            continue;
        }

        let height_map_data_cloned = height_map_data.as_ref().unwrap().clone();

        /*  let sub_heightmap = SubHeightMapU16::from_heightmap_u16(
                &height_map_data_cloned,
                height_map_subsection_pct
        ); */

        if let Some(mut entity_commands) = commands.get_entity(chunk_entity) {
            entity_commands
                .insert(ChunkCollisionData {
                    heightmap: height_map_data_cloned,
                })
                .insert(AddedChunkCollisionData {});
        }

        //println!(" spawning collision data entity for chunk ");

        //    }
    }
}
