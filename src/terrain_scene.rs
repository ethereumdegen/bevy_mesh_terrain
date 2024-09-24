

use avian3d::prelude::Collider;
use serde::Serialize;
use serde::Deserialize;

use bevy::utils::HashMap; 
use bevy::prelude::*;

use crate::pre_mesh::PreMesh;

/*

This is a serializeable binary file that stores data related to: 

height map   (one big array?)
splat map    (one big array?)




and BUILT data:  
-binary computed mesh data for each LOD of each chunk  (0,1,2)
-binary computed collision data for each chunk (same as above?)



i believe that stitch data wont be included here but will have to be computed at run time as it is highly dynamic and depends on the active LOD level of each chunk 




*/

#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct TerrainScene {

	pub height_map_array: Vec<Vec<u16>>,
	pub splat_map_array: Vec<Vec<u8>>,  //RGBA8 

	pub computed_scene_data:  ComputedTerrainSceneData 

}


impl TerrainScene {

	pub fn build_computed_scene_data(&mut self){



		// ... 
	}


}

#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct ComputedTerrainSceneData {

	pub computed_chunks_data: HashMap<u16, ComputedChunkSceneData >
 

}

pub type ComputedChunkSceneData = HashMap<u16, ComputedChunkLODSceneData > ;

 

#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct ComputedChunkLODSceneData { 
	pub chunk_lod_premesh: PreMesh,
	pub collider: Collider  // Collider::trimesh_from_mesh(&mesh)
}