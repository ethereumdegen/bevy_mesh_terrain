
use crate::TerrainMaterialExtension;
use crate::terrain::TerrainData;
use crate::terrain::TerrainImageDataLoadStatus;
use crate::terrain_config::TerrainConfig;
use std::path::PathBuf;
use std::path::Path;
use image::RgbaImage;
use image::ImageBuffer;
use crate::chunk::Chunk;
use crate::chunk::ChunkData;
use serde::Serialize;
use serde::Deserialize;
use bevy::prelude::*;
use bevy::utils::HashMap;

use bevy::render::render_resource::{ Extent3d, TextureDimension, TextureFormat};
use bevy::render::render_asset::RenderAssetUsages;
 

//this is what is loaded into memory as you are painting !! For less CPU effort
/*#[derive( Clone,Debug)]
pub struct LevelSplatDataRaw {

	//chunk id -> 
	pub splat_chunks: HashMap<u32, ChunkSplatDataRaw>

}*/



pub fn hypersplat_plugin(app:&mut App){


    app 
        .add_systems(Update, 
            (build_chunk_splat_data,
            rebuild_chunk_splat_textures 

            ).chain()

        )

    ;






}


//like a super PNG essentially 
#[derive(Component,Clone,Debug)]
pub struct ChunkSplatDataRaw {

	//pixel id -> 
	pub splat_pixels: Vec<Vec< SplatPixelDataRaw >>, //<u32, SplatPixelDataRaw>
    pub pixel_dimensions: UVec2
}

impl ChunkSplatDataRaw {

    pub fn set_exact_pixel_data(
        &mut self, 
        x:u32,
        y:u32,
        texture_type_index: u8,
        texture_strength: u8 

        ){  

        info!("setting exact pixel data {} {} {} {}", x,y,texture_type_index,texture_strength);

        self.splat_pixels[y as usize][x as usize].set_exact_pixel_data(
            texture_type_index,
            texture_strength
            );

    }

    pub fn clear_all_pixel_data(
        &mut self, 
        x:u32,
        y:u32) {

         self.splat_pixels[y as usize][x as usize] = SplatPixelDataRaw::new();
    }

    
     pub fn build_from_images(
        splat_index_map: &Image,
        splat_strength_map: &Image
    ) -> Self {
        // Extract dimensions of the images (assuming both images have the same dimensions)
        let width = splat_index_map.texture_descriptor.size.width;
        let height = splat_index_map.texture_descriptor.size.height;

        let pixel_dimensions = UVec2::new(width, height);

        // Initialize the pixel array to the correct size
        let mut splat_pixels = vec![
            vec![
                SplatPixelDataRaw {
                    pixel_data: HashMap::new()
                };
                width as usize
            ];
            height as usize
        ];

        // Raw data from both images
        let splat_index_data = &splat_index_map.data; // vec u8 
        let splat_strength_data = &splat_strength_map.data;  // vec u8 

        // Each pixel in the splat index map has 4 channels (RGBAUint8)
        // Each pixel in the splat strength map has 4 channels, but each channel is 2 bytes (u8)
        let index_channels = 4; // RGBA (4 channels, 1 byte each)
        let strength_channels = 4; // RGBA for material strength (but each channel is 2 bytes, so 8 bytes total per pixel)

        // Iterate through each pixel and populate the splat_pixels array
        for y in 0..height {
            for x in 0..width {
                let pixel_index = (y * width + x) as usize;

                // Extract the index and strength data for the current pixel
                let index_offset = pixel_index * index_channels;
                let strength_offset = pixel_index * strength_channels ; // Each strength value is 2 bytes 

                let mut splat_pixel_data = SplatPixelDataRaw {
                    pixel_data: HashMap::new(),
                };

                // Loop over each of the 4 channels (RGBA layout) for both index and strength maps
                for i in 0..index_channels {
                    let texture_type_index = splat_index_data[index_offset + i];

                     let texture_strength = splat_strength_data[strength_offset + i];
                    
                    

                    // Only store valid texture indices (non-zero material indices with strength)
                    if texture_strength > 0 {
                        splat_pixel_data.set_exact_pixel_data(
                            texture_type_index, 
                            texture_strength
                        );
                    }
                }

                // Set the pixel data at the current x, y coordinates
                splat_pixels[y as usize][x as usize] = splat_pixel_data;
            }
        }

        Self {
            splat_pixels,
            pixel_dimensions,
        }
    }


}



//this can actually describe how a great number of materials (more than 4)
// are applied to the terrain - thus better than a .PNG pixel 
#[derive( Clone,Debug)]
pub struct SplatPixelDataRaw {

 //	material_layer_id index  ->  strength 
 // when this is edited, make sure to always keep it sorted ! (?)
	pub pixel_data: HashMap<u8, u8>




}


impl SplatPixelDataRaw {

    fn new() -> Self{

        Self{
            pixel_data: HashMap::new()
        }
    }

    fn set_exact_pixel_data(
        &mut self,
        texture_type_index:u8,
        texture_strength:u8,
    ){

        // info!("setting exact pixel data  {} {}",  texture_type_index,texture_strength);


        self.pixel_data.insert(
            texture_type_index.into(), 
            texture_strength
            ) ;
 
    }

    

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

/*
#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct LevelSplatData {

	//chunk id -> 
	pub splat_chunks: HashMap<u32, ChunkSplatData>

}*/

#[derive(Component)]
pub struct SplatMapHandlesNeedReload;


#[derive(Serialize,Deserialize,Clone,Debug,Component)]
pub struct ChunkSplatData {
 
    pub splat_pixels: Vec<Vec< SplatPixelData  >>, 
    pub pixel_dimensions: UVec2


}

/*

pub struct SplatPixelData {
 
    pub material_index_array: [u8; 4],        
    pub material_strength_array: [u8; 4],

}

*/
impl From<ChunkSplatDataRaw> for ChunkSplatData {

 fn from(chunk_splat_data_raw: ChunkSplatDataRaw) -> Self {
        // Convert the 2D Vec<Vec<SplatPixelDataRaw>> to Vec<Vec<SplatPixelData>>
        let splat_pixels: Vec<Vec<SplatPixelData>> = chunk_splat_data_raw
            .splat_pixels
            .into_iter()
            .map(|row| {
                row.into_iter()
                    .map(SplatPixelData::from) // Use the From<SplatPixelDataRaw> implementation
                    .collect()
            })
            .collect();

        ChunkSplatData {
            splat_pixels,
            pixel_dimensions: chunk_splat_data_raw.pixel_dimensions, // Copy the pixel dimensions
        }
    }
}


impl ChunkSplatData{

    //builds an RGBAUint8  image for the index map  and an  RGBAsrgb (float)  image for the strength map 
      pub fn build_images(&self) -> (Image, Image) {
        // Create buffers for index and strength maps
        let width = self.pixel_dimensions.x;
        let height = self.pixel_dimensions.y;

        // Buffers to hold pixel data for each image
        let mut index_map_data = Vec::with_capacity((width * height * 4) as usize); // RGBA, 8-bit unsigned integer
        let mut strength_map_data = Vec::with_capacity((width * height * 8) as usize); // RGBA, 16-bit float (2 bytes per channel)

        // Fill in the pixel data from splat_pixels
         for y in 0..height {
            for x in 0..width {
                let pixel = &self.splat_pixels[y as usize][x as usize];

                // Extract the material indices and strengths from the pixel
                let material_indices = &pixel.material_index_array;
                let material_strengths = &pixel.material_strength_array;

                // Add the 4 channels (RGBA) for the index map
                index_map_data.push(material_indices[0]);
                index_map_data.push(material_indices[1]);
                index_map_data.push(material_indices[2]);
                index_map_data.push(material_indices[3]);

                // Add the 4 channels (RGBA) for the strength map
                strength_map_data.push(material_strengths[0]);
                strength_map_data.push(material_strengths[1]);
                strength_map_data.push(material_strengths[2]);
                strength_map_data.push(material_strengths[3]);
            }
        }
        info!("index map data length {}", index_map_data.len());
        info!("strength map data length {}", strength_map_data.len());

        // Create the index map image (RGBA8Uint format)
        let index_map = Image::new(
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            index_map_data,
            TextureFormat::Rgba8Uint, // Index map uses unsigned integers for material indices
            RenderAssetUsages::default(),
        );

        // Create the strength map image (RGBA8Uint format)
        let strength_map = Image::new(
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            strength_map_data, // 
            TextureFormat::Rgba8Uint, //  
            RenderAssetUsages::default(),
        );

        (index_map, strength_map)
    }

}

//this ends up being able to produce our 2 Control Maps which we will send to our GPU shader 
// then, our shader is going to have to do up to 4  UV texture lookups at this pixel  and combine them together..
#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct SplatPixelData {
 
	pub material_index_array: [u8; 4],        // Changed from usize to u32
    pub material_strength_array: [u8; 4],

}

impl From<SplatPixelDataRaw> for SplatPixelData {
    fn from(pixel_data_raw: SplatPixelDataRaw) -> Self {
        // Sort the pixel_data by strength (u8) in descending order
        let mut sorted_pixel_data: Vec<(u8, u8)> = pixel_data_raw
            .pixel_data
            .into_iter()
            .collect();

        // Sort by the u16 strength value in descending order
        sorted_pixel_data.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take the top 4 materials, or default to (0, 0.0) if less than 4
        let mut material_index_array = [0; 4];
        let mut material_strength_array = [0; 4];

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



fn build_chunk_splat_data(
    mut commands:Commands, 

    chunk_query: Query<(Entity, &Chunk, & ChunkData,& ChunkSplatDataRaw), 
     Changed<ChunkSplatDataRaw>    >, 

){ 

      for (entity, chunk, chunk_data, chunk_splat_data_raw ) in chunk_query.iter() {


          if let Some(mut cmd) = commands.get_entity(  entity ) {

                let chunk_splat_data: ChunkSplatData = chunk_splat_data_raw.clone().into();

                  cmd.try_insert( 
                   chunk_splat_data
                 );
          }


      }


}


fn rebuild_chunk_splat_textures(
  //   mut commands:Commands,

     mut chunk_query: Query<(Entity, &Chunk, &mut ChunkData,& ChunkSplatData, &Parent ), 
       Changed<ChunkSplatData >    >, 

     terrain_query: Query<(&TerrainData, &TerrainConfig)>,

     mut terrain_materials: ResMut<Assets<TerrainMaterialExtension>>,

     asset_server: Res<AssetServer>, 


    ){


    for (entity, chunk, mut chunk_data, chunk_splat_data, parent_terrain_entity ) in chunk_query.iter_mut() { 



          let terrain_entity_id = parent_terrain_entity.get();

            if terrain_query.get(terrain_entity_id).is_ok() == false {
                continue;
            }

            let (terrain_data, terrain_config) = terrain_query.get(terrain_entity_id).unwrap();
 


                     //   let file_name = format!("{}.png", chunk.chunk_id);
                     //   let asset_folder_path = PathBuf::from("assets");

                        let (chunk_splat_index_map_image,chunk_splat_strength_map_image) 
                                = chunk_splat_data.build_images();
                             
                      

                        //let chunk_splat_index_map_handle = &chunk_data.splat_index_texture_handle ;
                       // let chunk_splat_strength_map_handle = &chunk_data.splat_strength_texture_handle ;


                        let chunk_splat_index_texture  = asset_server.add( chunk_splat_index_map_image );
                        let chunk_splat_strength_texture  = asset_server.add( chunk_splat_strength_map_image );

                        chunk_data.splat_index_texture_handle = Some(chunk_splat_index_texture.clone());
                        chunk_data.splat_strength_texture_handle = Some(chunk_splat_strength_texture.clone());


                        if let Some( terrain_material_handle ) = &  chunk_data.material_handle {
                            if let Some(terrain_material) = terrain_materials.get_mut( terrain_material_handle ){


                             terrain_material.extension.splat_index_map_texture = Some(chunk_splat_index_texture.clone() );
                             terrain_material.extension.splat_strength_map_texture = Some(chunk_splat_strength_texture.clone());
                       
                            }


                        }
                        
                       /* save_chunk_splat_index_map_to_disk(
                            &chunk_splat_index_map_image,
                            asset_folder_path
                                .join(&terrain_config.splat_folder_path)
                                .join("index_maps")
                                .join(&file_name),
                        );

                        save_chunk_splat_strength_map_to_disk(
                                &chunk_splat_strength_map_image,
                                asset_folder_path
                                    .join(&terrain_config.splat_folder_path)
                                    .join("strength_maps")
                                    .join(&file_name),
                            );*/
                        


                       // if let Some(mut cmd) = commands.get_entity(entity){
                       //     cmd.try_insert( SplatMapHandlesNeedReload );
                      //  }

                        

    }

}

/*
fn rebuild_chunk_splat_textures(
    mut commands:Commands, 

    chunk_query: Query<(Entity, &Chunk, & ChunkData), 
     Changed<ChunkSplatData>    >,
 

){


     for (entity, chunk, chunk_data, chunk_splat_data_raw ) in chunk_query.iter() {


          if let Some(mut cmd) = commands.get_entity(  entity ) {

                  cmd.try_insert( 
                    ChunkSplatData ::build_from_raw(
                       chunk_splat_data_raw
                    ) 
                 );
          }


      }
 


}*/



// ----


pub fn save_chunk_splat_index_map_to_disk<P>(splat_image: &Image, save_file_path: P)
where
    P: AsRef<Path> + Clone,
{
    // Attempt to find the image in the Assets<Image> collection

    // Assuming the image format is Rgba8, which is common for splat maps
    let image_data = &splat_image.data;
    // Create an image buffer from the raw image data
    let format = splat_image.texture_descriptor.format;
    let width = splat_image.texture_descriptor.size.width;
    let height = splat_image.texture_descriptor.size.height;

    // Ensure the format is Rgba8 or adapt this code block for other formats
    if format == TextureFormat::Rgba8Uint  
    //   || format == TextureFormat::Rgba16Unorm
    {
        // The data in Bevy's Image type is stored in a Vec<u8>, so we can use it directly
        let img: RgbaImage = ImageBuffer::from_raw(width, height, image_data.clone())
            .expect("Failed to create image buffer");

        // Save the image to the specified file path
        img.save(&save_file_path).expect("Failed to save splat map");
        println!("saved splat image {}", save_file_path.as_ref().display());
    } else {
        eprintln!("Unsupported image format for saving chunk_splat_index_map: {:?}", format);
    }
}


pub fn save_chunk_splat_strength_map_to_disk<P>(splat_image: &Image, save_file_path: P)
where
    P: AsRef<Path> + Clone,
{
    // Attempt to find the image in the Assets<Image> collection

    // Assuming the image format is Rgba8, which is common for splat maps
    let image_data = &splat_image.data;
    // Create an image buffer from the raw image data
    let format = splat_image.texture_descriptor.format;
    let width = splat_image.texture_descriptor.size.width;
    let height = splat_image.texture_descriptor.size.height;



    // Ensure the format is Rgba8 or adapt this code block for other formats
    if format == TextureFormat::Rgba8Uint // || format == TextureFormat::Rgba8UnormSrgb
    //   || format == TextureFormat::Rgba16Unorm
    {
        // The data in Bevy's Image type is stored in a Vec<u8>, so we can use it directly
        let img: RgbaImage = ImageBuffer::from_raw(width, height, image_data.clone())
            .expect("Failed to create image buffer");

        // Save the image to the specified file path
        img.save(&save_file_path).expect("Failed to save splat map");
        println!("saved splat image {}", save_file_path.as_ref().display());
    } else {
        eprintln!("Unsupported image format for saving chunk_splat_strength_map: {:?}", format);
    }
}