
use bevy::{prelude::*  };

use crate::{chunk::{Chunk, ChunkEvent}, terrain::{TerrainData, TerrainConfig}};

/*

use parry3d  heightfield  collider 

get the data from the heightmap u16 ! to build a heightfield collider 
Can spawn a collider on each entity that has the 'Chunk' component... the collider for that chunk  

*/


/*
It is the end users responsibility to detect for these components and REPLACE them with their own collider component 

finish me !!! This should easily plug in to bevy xpbd 
*/ 
#[derive(Component)]
pub struct CollisionData {
     
     //add heightfield 
     
     //dont need translation since its on entity alrdy ...
}
 

  
pub fn spawn_chunk_collision_data(
    mut commands: Commands, 
     
    mut chunk_query: Query<(Entity,  &mut Chunk, &Parent), Without<CollisionData>  > ,
    
    mut terrain_query : Query<(&mut TerrainData,&TerrainConfig, &Transform)> ,
 
){ 
                
           for  ( chunk_entity, chunk_data, parent_terrain_entity  ) in  chunk_query.iter() { 
                
                    //let chunk_id = chunk_data.chunk_id;  
                    
                    if let Ok( (mut terrain_data, terrain_config, terrain_transform ) ) = terrain_query.get_mut( parent_terrain_entity.get() ){
                        
                        commands.entity( chunk_entity).insert(  
                            CollisionData {}                            
                        );
                        //println!(" spawning collision data entity for chunk ");
                        
                     }
                
                
                
            }  
           
        
        
}