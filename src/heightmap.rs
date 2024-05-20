use std::io::BufWriter;
use std::fs::File;
use std::path::Path;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;

use thiserror::Error;

/*
https://github.com/norman784/gaiku/blob/master/crates/gaiku_baker_heightmap/src/lib.rs
*/

#[derive(Error, Debug)]
pub enum HeightMapError {
    #[error("failed to load the image")]
    LoadingError,
}

pub type HeightMapU16 = Vec<Vec<u16>>;

 

 // height data is in format [y][x]  

pub trait HeightMap {
    fn load_from_image(image: &Image) -> Result<Box<Self>, HeightMapError>;
    fn save_heightmap_to_image<P>(&self,  save_file_path: P ) where  P:AsRef<Path>;

      fn append_x_row(&mut self, row: Vec<u16>);
    fn append_y_col(&mut self, col: Vec<u16>);

}


impl HeightMap for HeightMapU16 {
    fn load_from_image(image: &Image) -> Result<Box<Self>, HeightMapError> {
        let width = image.size().x as usize;
        let height = image.size().y as usize;

        let format = image.texture_descriptor.format;

        if format != TextureFormat::R16Uint {
            println!("heightmap: wrong format {:?}", format);
            return Err(HeightMapError::LoadingError);
        }
 


        let mut height_map = Vec::with_capacity(height);
        let mut data_iter = image.data.chunks_exact(2);

        for _ in 0..height {
            let mut row = Vec::with_capacity(width);

            for _ in 0..width {
                if let Some([b1, b2]) = data_iter.next() {
                    let height_value = u16::from_le_bytes([*b1, *b2]);
                    row.push(height_value);
                } else {
                    println!("heightmap: unexpected end of data");
                    return Err(HeightMapError::LoadingError);
                }
            }

            height_map.push(row);
        }



        Ok(Box::new(height_map))
    }


    fn save_heightmap_to_image<P>(&self,  save_file_path: P ) where  P: AsRef<Path>,
    { 
  
       let chunk_height_data = self ;

        // Assuming chunk_height_data is a Vec<Vec<u16>>
        let height = chunk_height_data.len();
        let width = chunk_height_data.first().map_or(0, |row| row.len());

        // Prepare the file and writer
        let file = File::create(save_file_path).expect("Failed to create file");
        let ref mut w = BufWriter::new(file);

        // Set up the encoder. Since PNG is the format that supports 16-bit grayscale natively, we use it here.
        let mut encoder = png::Encoder::new(w, width as u32, height as u32); // Width and height of image
        encoder.set_color(png::ColorType::Grayscale);
        encoder.set_depth(png::BitDepth::Sixteen);
        let mut writer = encoder.write_header().expect("Failed to write PNG header");

        // Flatten the Vec<Vec<u16>> to a Vec<u8> for the PNG encoder
        let mut buffer: Vec<u8> = Vec::with_capacity(width * height * 2);
        for row in chunk_height_data {
            for value in row {
                buffer.extend_from_slice(&value.to_be_bytes()); // Ensure big-endian byte order
            }
        }

        // Write the image data
        writer
            .write_image_data(&buffer)
            .expect("Failed to write PNG data");

    }


    fn append_x_row(&mut self, row: Vec<u16>) {
        self.push(row);
    }

    fn append_y_col(&mut self, col: Vec<u16>) {
        if col.len() != self.len() {
            println!("WARN: cannot append y col");
            panic!("Column length does not match the number of rows in height data.");
        }

        for (row, &value) in self.iter_mut().zip(col.iter()) {
            row.push(value);
        }
    }
   
}
