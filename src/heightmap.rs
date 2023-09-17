

/*
https://github.com/norman784/gaiku/blob/master/crates/gaiku_baker_heightmap/src/lib.rs
*/

use bevy::{prelude::*, render::render_resource::TextureFormat};
use thiserror::Error;

 

#[derive(Error,Debug)]
pub enum HeightMapError{
    #[error("failed to load the image")]
    LoadingError
    
}

pub type HeightMapU16 = Vec<Vec<u16>>;
 

pub trait HeightMap {
    fn load_from_image( image: &Image ) ->   Result<Box<Self>,HeightMapError>  ; 
      
}
 


impl HeightMap for HeightMapU16 {
    fn load_from_image(image:  &Image) ->    Result<Box<Self>,HeightMapError>   {
        
        let width = image.size().x as usize;
        let height = image.size().y as usize;
        
        let format = image.texture_descriptor.format;
        
        if format != TextureFormat::R16Uint {
           return Err( HeightMapError::LoadingError  );
        }
        
        //maybe somehow fail if the format is not R16uint 
        
       // With the format being R16Uint, each pixel is represented by 2 bytes
        let mut height_map = Vec::with_capacity(height);
        
        for y in 0..height {
            let mut row = Vec::with_capacity(width);
            for x in 0..width {
                let index = 2 * (y * width + x); // 2 because of R16Uint
                let height_value = u16::from_le_bytes([image.data[index], image.data[index + 1]]);
                row.push(height_value);
            }
            height_map.push(row);
        }

       Ok(Box::new(  height_map  ) )
        
    }
    
    /*
      fn to_collider_heightmap(&self) -> HeightField {
        
         Collider::heightfield(
                    heightmap.heightmap.clone(), 
                    heightmap.scale.clone()            
                    )
                    
    }*/
    
}
  