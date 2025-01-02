
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

 
}

impl ChunkSplatDataRaw {

   


    fn get_pixel_internal_index(
        x:u32,
        y:u32,
        layer:u8 , 
        width: u32
    ) -> usize {


         let layers_count = 4; 

         if layer > layers_count {
            warn!("invalid layer ! {}", layer);
         }

         //let width = self.splat_index_map_texture.width();

         let pixel_index = (y * width + x) as usize;

                // Extract the index and strength data for the current pixel
         let index_offset = pixel_index * layers_count as usize;

         let idx = index_offset + layer as usize; 

         idx
    }


    pub fn set_pixel_index_map_data(
        &mut self, 
        x:u32,
        y:u32,
        layer:u8 , 
        texture_type_index: u8,
        

        ){  

        

        //layer must be 0,1,2 or 4 and that is RGBA respectively 

           

        let width = self.splat_index_map_texture.width();
         
        let idx = Self::get_pixel_internal_index(x,y,layer,width); 

        self.splat_index_map_texture.data[idx] = texture_type_index;       

    }


    pub fn set_pixel_strength_map_data(
        &mut self, 
        x:u32,
        y:u32,
        layer:u8 , 
        
        texture_strength: u8 

        ){  

        

        //layer must be 0,1,2 or 3 and that is RGBA respectively 

        let width = self.splat_index_map_texture.width();
         
        let idx = Self::get_pixel_internal_index(x,y,layer,width); 
        
        self.splat_strength_map_texture.data[idx] = texture_strength;


    }


      pub fn get_pixel_index_map_data(
        &mut self, 
        x:u32,
        y:u32,
        layer:u8  
        ) -> u8 {  
 

        //layer must be 0,1,2 or 3 and that is RGBA respectively 

        let width = self.splat_index_map_texture.width();
         
        let idx = Self::get_pixel_internal_index(x,y,layer,width); 

         self.splat_index_map_texture.data[idx]      

    }

     pub fn get_pixel_strength_map_data(
        &mut self, 
        x:u32,
        y:u32,
        layer:u8  
        ) -> u8 {  
 

        //layer must be 0,1,2 or 3 and that is RGBA respectively 

        let width = self.splat_index_map_texture.width();
         
        let idx = Self::get_pixel_internal_index(x,y,layer,width); 

         self.splat_strength_map_texture.data[idx]      

    }

    pub fn clear_all_pixel_data(
        &mut self, 
        x:u32,
        y:u32 ) {


             let width = self.splat_index_map_texture.width();

           for layer in 0..3 { 
                 //self.splat_pixels[layer as usize][y as usize][x as usize] = SplatPixelDataRaw::new();


  
         
                 let idx = Self::get_pixel_internal_index(x,y,layer,width); 



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
 
                info!("replacing splat   texture " );

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


                        //try to do this later, when its actually loaded , to remove flickering 
                    /*    if let Some(terrain_material_handle)= &chunk_data.material_handle {
                             if let Some( terrain_material ) = terrain_materials.get_mut ( terrain_material_handle )  {

 
                                 terrain_material.extension.splat_index_map_texture =  chunk_data.splat_index_texture_handle.clone() ;
                                 terrain_material.extension.splat_strength_map_texture =  chunk_data.splat_strength_texture_handle.clone() ;


                            }

                        }*/
                        
                        
                        

    }

}
 


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


    info!("saving splat image {} {} ", &width  ,   &height );

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

      info!("saving splat image {} {} ", &width  ,   &height );

    // Ensure the format is Rgba8 or adapt this code block for other formats
    if format == TextureFormat::Rgba8Unorm // || format == TextureFormat::Rgba8UnormSrgb
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