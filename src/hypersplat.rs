
use serde::Serialize;
use serde::Deserialize;
use bevy::prelude::*;
use bevy::utils::HashMap;


#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct LevelSplatDataRaw {

	//chunk id -> 
	pub splat_chunks: HashMap<u32, ChunkSplatDataRaw>

}



//like a super PNG essentially 
#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct ChunkSplatDataRaw {

	//pixel id -> 
	pub splat_pixels: HashMap<u32, SplatPixelDataRaw>

}



//this can actually describe how a great number of materials (more than 4)
// are applied to the terrain - thus better than a .PNG pixel 
#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct SplatPixelDataRaw {

 //	material_layer_id -> 
 // when this is edited, make sure to always keep it sorted ! (?)
	pub pixel_data: HashMap<u32, f32>




}


// ------------


/*

Will need to produce a few control maps (TGA or PNG) for the gpu...

A. MaterialIndexControlMap : One of the control maps will actually tell the GPU, per pixel, which of the 4 control maps to use and in which order 

(  r value ->  the control map index for layer 0 to use for this pixel   )
(  g value ->  the control map index for layer 1 to use for this pixel   )
(  b value ->  the control map index for layer 2 to use for this pixel   )
(  a value ->  the control map index for layer 3 to use for this pixel   )


B) MaterialStrengthControlMap : The second control map will tell the GPU how much exposure to give each layer (transparency)

( r value -> The lerp amount / power for layer 0 at this pixel)
( g value -> The lerp amount / power for layer 1 at this pixel )
( b value -> The lerp amount / power for layer 2 at this pixel )
( a value -> The lerp amount / power for layer 3 at this pixel )


*/

#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct LevelSplatData {

	//chunk id -> 
	pub splat_chunks: HashMap<u32, ChunkSplatData>

}


#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct ChunkSplatData {

	//pixel id -> 
	pub splat_pixels: HashMap<u32, SplatPixelData>

}


//this ends up being able to produce our 2 Control Maps which we will send to our GPU shader 
// then, our shader is going to have to do up to 4  UV texture lookups at this pixel  and combine them together..
#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct SplatPixelData {
 
	pub material_index_array: [u32; 4],        // Changed from usize to u32
    pub material_strength_array: [f32; 4],

}

impl From<SplatPixelDataRaw> for SplatPixelData {
    fn from(pixel_data_raw: SplatPixelDataRaw) -> Self {
        // Sort the pixel_data by strength (f32) in descending order
        let mut sorted_pixel_data: Vec<(u32, f32)> = pixel_data_raw
            .pixel_data
            .into_iter()
            .collect();

        // Sort by the f32 strength value in descending order
        sorted_pixel_data.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take the top 4 materials, or default to (0, 0.0) if less than 4
        let mut material_index_array = [0; 4];
        let mut material_strength_array = [0.0; 4];

        for (i, (material_index, strength)) in sorted_pixel_data.into_iter().take(4).enumerate() {
            material_index_array[i] = material_index;
            material_strength_array[i] = strength;
        }

        SplatPixelData {
            material_index_array,
            material_strength_array,
        }
    }
}
