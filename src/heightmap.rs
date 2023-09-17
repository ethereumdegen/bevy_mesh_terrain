

/*
https://github.com/norman784/gaiku/blob/master/crates/gaiku_baker_heightmap/src/lib.rs
*/

use bevy::prelude::*;
  

pub type HeightMapU16 = Vec<Vec<u8>>;


pub trait HeightMap {
    fn load_from_image( image: &Image ) -> Self ;
}
 


impl HeightMap for HeightMapU16 {
    fn load_from_image(image:  &Image) -> Self {
        
        let width = image.size().x as usize;
        let height = image.size().y as usize;
        
        // Assuming the format is RGBA, let's extract the red channel as our height values
        let mut height_map = Vec::with_capacity(height);
        
        for y in 0..height {
            let mut row = Vec::with_capacity(width);
            for x in 0..width {
                let index = 4 * (y * width + x); // 4 because of RGBA
                let height_value = image.data[index];
                row.push(height_value);
            }
            height_map.push(row);
        }

        height_map
    }
}
 