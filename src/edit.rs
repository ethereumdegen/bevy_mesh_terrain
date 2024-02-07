use std::ops::{Add, Div, Neg};

use bevy::ecs::entity::Entity;
use bevy::math::Vec2;

use bevy::ecs::event::Event;
use bevy::prelude::EventReader;

use bevy::asset::{AssetServer, Assets};
use bevy::render::render_resource::{Extent3d, TextureFormat};
use bevy::render::texture::Image;

use bevy::prelude::*;

use crate::chunk::{Chunk, ChunkData, ChunkHeightMapResource};
use crate::terrain::{TerrainData, TerrainImageDataLoadStatus};
use crate::terrain_material::TerrainMaterial;

#[derive(Debug)]
pub enum EditingTool {
    SetHeightMap(u16, f32, bool),       // height, radius, save to disk
    SetSplatMap(u8, u8, u8, f32, bool), //R, G, B, radius, save to disk
}

// entity, editToolType, coords, magnitude
#[derive(Event)]
pub struct EditTerrainEvent {
    pub entity: Entity,
    pub tool: EditingTool,
    pub coordinates: Vec2,
}

#[derive(Event)]
pub enum TerrainCommandEvent {
    SaveAllChunks,
}

pub fn apply_tool_edits(
    mut asset_server: Res<AssetServer>,

    mut chunk_query: Query<(&Chunk, &mut ChunkData, &Parent)>, //chunks parent should have terrain data
    chunk_mesh_query: Query<(&Parent, &GlobalTransform)>,

    mut images: ResMut<Assets<Image>>,
    mut terrain_materials: ResMut<Assets<TerrainMaterial>>,

    mut chunk_height_maps: ResMut<ChunkHeightMapResource>,

    mut ev_reader: EventReader<EditTerrainEvent>,
) {
    for ev in ev_reader.read() {
        eprintln!("-- {:?} -- terrain edit event!", &ev.tool);

        let intersected_entity = &ev.entity;

        //  if let Some((chunk, mut chunk_data)) = chunk_query.get_mut(intersected_entity.clone()).ok()
        if let Some((chunk_entity, chunk_transform)) =
            chunk_mesh_query.get(intersected_entity.clone()).ok()
        {
            if let Some((chunk, mut chunk_data, terrain_entity)) =
                chunk_query.get_mut(chunk_entity.get().clone()).ok()
            {
                //   if let Some(mut terrain_data) = terrain_data_query.get_mut(terrain_entity.get().clone()).ok() { //why cant i find this ?

                let chunk_dimensions = Vec2::new(256.0, 256.0); //compute me from config

                match &ev.tool {
                    EditingTool::SetHeightMap(height, radius, save_to_disk) => {
                        if let Some(height_map_data) =
                            &mut chunk_height_maps.chunk_height_maps.get_mut(&chunk.chunk_id)
                        {
                            // if let Some(img) = images.get_mut( height_map_image_handle ){

                            let tool_coords: &Vec2 = &ev.coordinates;
                            let chunk_transform = chunk_transform.translation();
                            let chunk_transform_vec2: Vec2 =
                                Vec2::new(chunk_transform.x, chunk_transform.z);

                            let chunk_center_transform =
                                chunk_transform_vec2.add(chunk_dimensions.div(2.0));

                            let chunk_local_distance = tool_coords.distance(chunk_center_transform);

                            let tool_coords_local = tool_coords.add(chunk_transform_vec2.neg());

                            //need to make an array of all of the data indices of the terrain that will be set .. hm ?

                            let img_data_length = height_map_data.0.len();
                            //println!("trying to edit the height map via its handle :)  {}", max );

                            // let mut idx_array: [usize; 2];

                            let radius_clone = radius.clone();
                            //fake it for now
                            for x in 0..img_data_length {
                                for y in 0..img_data_length {
                                    let local_coords = Vec2::new(x as f32, y as f32);

                                    if tool_coords_local.distance(local_coords) < radius_clone {
                                        height_map_data.0[x][y] = height.clone();
                                    }
                                }
                            }

                            chunk_data.height_map_image_data_load_status =
                                TerrainImageDataLoadStatus::NeedsReload;
                        }
                    }

                    EditingTool::SetSplatMap(r, g, b, radius, save_to_disk) => {
                        if let Some(splat_image_handle) = chunk_data.get_splat_texture_image() {
                            if let Some(img) = images.get_mut(splat_image_handle) {
                                // Calculate the pixel position and radius in pixels
                                let img_size = img.size();

                                let tool_coords: &Vec2 = &ev.coordinates;
                                let chunk_transform = chunk_transform.translation();
                                let chunk_transform_vec2: Vec2 =
                                    Vec2::new(chunk_transform.x, chunk_transform.z);

                                let chunk_center_transform =
                                    chunk_transform_vec2.add(chunk_dimensions.div(2.0));

                                let chunk_local_distance =
                                    tool_coords.distance(chunk_center_transform);

                                let tool_coords_local = tool_coords.add(chunk_transform_vec2.neg());

                                let pixel_pos = Vec2::new(
                                    tool_coords_local.x / chunk_dimensions.x * img_size.x as f32,
                                    tool_coords_local.y / chunk_dimensions.y * img_size.y as f32,
                                );
                                let pixel_radius = *radius as f32;

                                println!("set splat map at {} {}", pixel_pos, pixel_radius);
                                // Assuming the image format is Rgba8
                                if img.texture_descriptor.format == TextureFormat::Rgba8Unorm
                                    || img.texture_descriptor.format
                                        == TextureFormat::Rgba8UnormSrgb
                                {
                                    //                let img_data = img.data.as_mut_slice();

                                    // Iterate over each pixel in the image
                                    for y in 0..img_size.y {
                                        for x in 0..img_size.x {
                                            let idx = (y * img_size.x + x) as usize * 4; // 4 bytes per pixel (R, G, B, A)
                                            let pixel_coords = Vec2::new(x as f32, y as f32);

                                            //  img.data[idx] = *r as u8;

                                            // Check if the pixel is within the tool's radius
                                            if pixel_coords.distance(pixel_pos) < pixel_radius {
                                                // Modify the pixel data
                                                img.data[idx] = *r as u8; // R
                                                img.data[idx + 1] = *g as u8; // G
                                                img.data[idx + 2] = *b as u8; // B
                                                                              // Alpha value remains unchanged

                                                println!("modify pixel data ");
                                            }
                                        }
                                    }

                                    // Mark the image as modified -- how does that work ? touch in another way ?
                                    /*img.texture_descriptor.size = Extent3d{
                                        width: img_size.x,
                                        height: img_size.y,
                                        depth_or_array_layers: 1,
                                    };*/

                                    let updated_image = img.clone();

                                    let updated_image_handle = asset_server.add(updated_image);

                                    chunk_data
                                        .set_splat_texture_image(updated_image_handle.clone()); //is this necessary? i think so in case the height is modified

                                    if let Some(material_handle) = &chunk_data.material_handle {
                                        if let Some(terrain_material) =
                                            terrain_materials.get_mut(material_handle)
                                        {
                                            //this should let us avoid rebuilding the entire mesh
                                            terrain_material.splat_texture =
                                                Some(updated_image_handle);
                                            println!("rewrote splat tex in terrain material ");
                                        }
                                    }

                                    //mark  material as needing reload !!
                                } else {
                                    println!(
                                        "incorrect tex format {:?}",
                                        img.texture_descriptor.format
                                    );
                                }
                            }
                        }
                    } // SetSplatMap
                } //match
            }
        }
    }
}
