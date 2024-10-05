
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
 

 

pub fn hypersplat_plugin(app:&mut App){


    app 
        .add_systems(Update, 
            (  //build_chunk_splat_data,
            rebuild_chunk_splat_textures 

            ).chain()

        )

    ;






}

#[derive(Component)]
pub struct SplatMapDataUpdated ;


//like a super PNG essentially 
#[derive(Component,Clone,Debug)]
pub struct ChunkSplatDataRaw {

    pub splat_index_map_texture: Image ,
    pub splat_strength_map_texture:  Image, 

	//pixel id -> 
	//pub splat_pixels: Vec<Vec<Vec< SplatPixelDataRaw >>>, //<u32, SplatPixelDataRaw>
   // pub pixel_dimensions: UVec2
}

impl ChunkSplatDataRaw {

    pub fn set_exact_pixel_data(
        &mut self, 
        x:u32,
        y:u32,
        layer:u8 , 
        texture_type_index: u8,
        texture_strength: u8 

        ){  

        info!("setting exact pixel data {} {} {} {}", x,y,texture_type_index,texture_strength);

        /*
        self.splat_pixels[layer as usize][y as usize][x as usize].set_exact_pixel_data(
            texture_type_index,
            texture_strength
            );

        */



        //layer must be 0,1,2 or 4 and that is RGBA respectively 

        let layers_count = 4; 

         if layer > layers_count {
            warn!("invalid layer ! {}", layer);
         }

         let width = self.splat_index_map_texture.width();

         let pixel_index = (y * width + x) as usize;

                // Extract the index and strength data for the current pixel
         let index_offset = pixel_index * layers_count as usize;

         let idx = index_offset + layer as usize; 


        self.splat_index_map_texture.data[idx] = texture_type_index;
        self.splat_strength_map_texture.data[idx] = texture_type_index;


    }

    pub fn clear_all_pixel_data(
        &mut self, 
        x:u32,
        y:u32 ) {


            let layers_count = 4; 

           for layer in 0..3 { 
                 //self.splat_pixels[layer as usize][y as usize][x as usize] = SplatPixelDataRaw::new();


   
                     let width = self.splat_index_map_texture.width();

                     let pixel_index = (y * width + x) as usize;

                            // Extract the index and strength data for the current pixel
                     let index_offset = pixel_index * layers_count as usize;

                     let idx = index_offset + layer as usize; 



                 self.splat_index_map_texture.data[idx] = 0;
                 self.splat_strength_map_texture.data[idx] = 0;
             }


          
    }

    
     pub fn build_from_images(
        splat_index_map: &Image,
        splat_strength_map: &Image
    ) -> Self {

        Self {
            splat_index_map_texture: splat_index_map.clone(),
            splat_strength_map_texture: splat_strength_map.clone(),
        }
       
    }

 

    //builds an RGBAUint8  image for the index map  and an  RGBAsrgb (float)  image for the strength map 
      pub fn get_images(&self) -> (&Image, &Image) {
        
        (&self.splat_index_map_texture, &self.splat_strength_map_texture)
    }
}


//this can actually describe how a great number of materials (more than 4)
// are applied to the terrain - thus better than a .PNG pixel 
#[derive( Clone,Debug)]
pub struct SplatPixelDataRaw {

 //	material_layer_id index  ->  strength 
 // when this is edited, make sure to always keep it sorted ! (?)
	//pub pixel_data: HashMap<u8, u8>

    pub texture_index: u8, 
    pub strength: u8 ,




}


impl SplatPixelDataRaw {

    
    fn set_exact_pixel_data(
        &mut self,
        texture_type_index:u8,
        texture_strength:u8,
    ){

        // info!("setting exact pixel data  {} {}",  texture_type_index,texture_strength);


        self.texture_index = texture_type_index;
        self.strength = texture_strength;
 
    }

    

}
 

fn rebuild_chunk_splat_textures(
    mut commands:Commands,

     mut chunk_query: Query<(Entity, &Chunk, &mut ChunkData,& ChunkSplatDataRaw, &Parent ), 
       With<SplatMapDataUpdated >    >, 

     terrain_query: Query<(&TerrainData, &TerrainConfig)>,

     mut terrain_materials: ResMut<Assets<TerrainMaterialExtension>>,

     asset_server: Res<AssetServer>, 


    ){


    for (chunk_entity, chunk, mut chunk_data, chunk_splat_data, parent_terrain_entity ) in chunk_query.iter_mut() { 


         if let Some(mut cmds) = commands.get_entity( chunk_entity ){


                cmds.remove::<SplatMapDataUpdated>();



           }


          let terrain_entity_id = parent_terrain_entity.get();

            if terrain_query.get(terrain_entity_id).is_ok() == false {
                continue;
            }

            let (terrain_data, terrain_config) = terrain_query.get(terrain_entity_id).unwrap();
 


                     //   let file_name = format!("{}.png", chunk.chunk_id);
                     //   let asset_folder_path = PathBuf::from("assets");

                        let (chunk_splat_index_map_image,chunk_splat_strength_map_image) 
                                = chunk_splat_data.get_images();
                             
                      

                        //let chunk_splat_index_map_handle = &chunk_data.splat_index_texture_handle ;
                       // let chunk_splat_strength_map_handle = &chunk_data.splat_strength_texture_handle ;


                        let chunk_splat_index_texture  = asset_server.add( chunk_splat_index_map_image.clone() );
                        let chunk_splat_strength_texture  = asset_server.add( chunk_splat_strength_map_image.clone() );

                        chunk_data.splat_index_texture_handle = Some(chunk_splat_index_texture.clone());
                        chunk_data.splat_strength_texture_handle = Some(chunk_splat_strength_texture.clone());


                       
                        
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