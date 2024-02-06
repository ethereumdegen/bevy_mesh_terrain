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

pub struct SubHeightMapU16(pub Vec<Vec<u16>>);

impl SubHeightMapU16 {
    pub fn from_heightmap_u16(
        heightmap: &HeightMapU16,
        // start_bound: [ usize; 2 ],
        //  end_bound: [ usize; 2 ],
        bounds_pct: [[f32; 2]; 2],
    ) -> SubHeightMapU16 {
        let width = heightmap.len() - 0;
        let height = heightmap[0].len() - 0;

        // let start_bound = [ (width as f32 * bounds_pct[0][0]) as usize, (height as f32 * bounds_pct[0][1]) as usize  ];
        //let end_bound = [ (width as f32 * bounds_pct[1][0]) as usize , (height as f32 * bounds_pct[1][1]) as usize   ];

        let start_bound = [
            (width as f32 * bounds_pct[0][0]).ceil() as usize,
            (height as f32 * bounds_pct[0][1]).ceil() as usize,
        ];

        //really need to load 1 extra row than we normally would think we would... so here it is
        let end_bound = [
            (width as f32 * bounds_pct[1][0]).ceil() as usize + 1,
            (height as f32 * bounds_pct[1][1]).ceil() as usize + 1,
        ];

        let mut height_data = Vec::new();

        for x in start_bound[0]..end_bound[0] {
            if x >= width {
                continue;
            }

            let mut row = Vec::new();
            for y in start_bound[1]..end_bound[1] {
                if y >= height {
                    continue;
                }

                row.push(heightmap[x][y]);
            }
            height_data.push(row);
        }

        SubHeightMapU16(height_data)
    }
}

pub trait HeightMap {
    fn load_from_image(image: &Image) -> Result<Box<Self>, HeightMapError>;
}

impl HeightMap for HeightMapU16 {
    fn load_from_image(image: &Image) -> Result<Box<Self>, HeightMapError> {
        let width = image.size().x as usize;
        let height = image.size().y as usize;

        let format = image.texture_descriptor.format;

        if format != TextureFormat::R16Uint {
            return Err(HeightMapError::LoadingError);
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

        Ok(Box::new(height_map))
    }

    /*
      fn to_collider_heightmap(&self) -> HeightField {

         Collider::heightfield(
                    heightmap.heightmap.clone(),
                    heightmap.scale.clone()
                    )

    }*/
}
