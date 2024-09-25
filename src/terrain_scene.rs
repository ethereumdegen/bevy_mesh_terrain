

use std::io::Read;
use std::fs::File;
use std::path::PathBuf;
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


//#[derive(Serialize,Deserialize,Clone,Debug)]
//pub struct TerrainSceneComponent (TerrainScene);


#[derive(Serialize,Deserialize,Clone,Debug,Component,Default)]
pub struct TerrainScene {

	pub height_map_array: Vec<Vec<u16>>,
	pub splat_map_array: Vec<Vec<u8>>,  //RGBA8 

	pub computed_scene_data:  ComputedTerrainSceneData 

}


impl TerrainScene {


	pub fn create_or_load(
			path: &PathBuf
		) -> Self {

		// load -- ? 

		match Self::load_from_disk( path ){

			Some(s) => s,
			None => Self::default()
		}

	

	}

	pub fn build_computed_scene_data(&mut self){



		// ... 
	}

	/* pub fn save_to_disk(&self, foliage_data_files_path: &str) -> Result<(), String> {
        let scene_name = self.foliage_scene_name.clone();
        // Ensure the directory exists
        let full_file_path = format!("{}{}", foliage_data_files_path, scene_name);

        // Open the file for writing
        let file_result = File::create(full_file_path);

        match file_result {
            Ok(mut file) => {
                // Serialize the data to binary using bincode
                let encoded: Vec<u8> = match bincode::serialize(self) {
                    Ok(data) => data,
                    Err(e) => {
                        return Err(format!("Failed to serialize data: {}", e));
                    }
                };

                // Write the binary data to the file
                if let Err(e) = file.write_all(&encoded) {
                    return Err(format!("Failed to write data to file: {}", e));
                }

                Ok(())
            }
            Err(e) => Err(format!("Failed to create file: {}", e)),
        }
    }*/

    

    // This function loads the FoliageSceneData from disk
    pub fn load_from_disk( path: &PathBuf ) -> Option<Self> {

        let full_file_path = path.as_path().to_str().unwrap();

        // Open the file for reading
        let file_result = File::open(full_file_path);

        match file_result {
            Ok(mut file) => {
                let mut buffer = Vec::new();

                // Read the binary data from the file
                if let Err(e) = file.read_to_end(&mut buffer) {
                    eprintln!("Failed to read data from file: {}", e);
                    return None;
                }

                // Deserialize the binary data into FoliageSceneData
                match bincode::deserialize(&buffer) {
                    Ok(data) => Some(data),
                    Err(e) => {
                        eprintln!("Failed to deserialize data: {}", e);
                        None
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to open file: {}", e);
                None
            }
        }
    }


}

#[derive(Serialize,Deserialize,Clone,Debug,Default)]
pub struct ComputedTerrainSceneData {

	pub computed_chunks_data: HashMap<u16, ComputedChunkSceneData >
 

}

pub type ComputedChunkSceneData = HashMap<u16, ComputedChunkLODSceneData > ;

 

#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct ComputedChunkLODSceneData { 
	pub chunk_lod_premesh: PreMesh,
	pub collider: Collider  // Collider::trimesh_from_mesh(&mesh)
}