/*

this is loaded from a RON file


also should incorporate the paths to the height and splat folders for their texture handles...

*/
use bevy::prelude::*;

use serde::{Deserialize, Serialize};

use std::fs::File;
use std::io::Read;

#[derive(Component, Deserialize, Serialize, Clone)]
pub struct TerrainConfig {
    pub terrain_dimensions: Vec2,

    pub chunk_rows: u32,

    pub render_distance: f32,
    pub lod_distance: f32,

    pub lod_level_offset: u8,

    pub height_scale: f32,

    pub use_greedy_mesh: bool,

    pub texture_image_sections: u32,
    pub diffuse_folder_path: String,
    pub height_folder_path: String,
    pub splat_folder_path: String,
    pub collider_data_folder_path: String,
}

impl Default for TerrainConfig {
    fn default() -> Self {
        Self {
            // chunk_width: 64.0 ,
            terrain_dimensions: Vec2::new(1024.0, 1024.0), //this should match the heightmap dimensions... consider removing this var or changing how it fundamentally works .
            chunk_rows: 16, //making this too high produces too many materials which causes lag.  Too low and we cant LOD properly . 16 seems good .
            render_distance: 2000.0,
            lod_distance: 1000.0,

            lod_level_offset: 0,

            height_scale: 0.004, //for building the mesh

            use_greedy_mesh: false,
            texture_image_sections: 8,

            diffuse_folder_path: "diffuse".into(),
            height_folder_path: "height".into(),
            splat_folder_path: "splat".into(),
            collider_data_folder_path: "collider".into(),
        }
    }
}

impl TerrainConfig {
    pub fn load_from_file(file_path: &str) -> Result<TerrainConfig, ron::Error> {
        let mut file = File::open(file_path).expect("Failed to open file");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Failed to read file");
        Ok(ron::from_str(&contents)?)
    }

    pub fn set_render_distance(mut self, distance: f32) -> Self {
        self.render_distance = distance;
        self
    }

    pub fn set_lod_distance(mut self, distance: f32) -> Self {
        self.lod_distance = distance;
        self
    }

    pub fn get_chunk_dimensions(&self) -> Vec2 {
        let chunk_dimension_x = self.terrain_dimensions.x / self.chunk_rows as f32;
        let chunk_dimension_z = self.terrain_dimensions.y / self.chunk_rows as f32;

        Vec2::new(chunk_dimension_x, chunk_dimension_z)
    }

    pub fn get_max_render_distance(&self) -> f32 {
        return self.render_distance;
    }

    pub fn get_chunk_render_distance(&self) -> u32 {
        return self.render_distance as u32 / self.chunk_rows;
    }

    pub fn get_chunk_lod_distance(&self) -> f32 {
        return self.lod_distance;
    }
}
