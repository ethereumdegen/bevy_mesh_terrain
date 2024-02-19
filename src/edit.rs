use std::ops::{Add, Div, Neg};

use bevy::ecs::entity::Entity;
use bevy::math::Vec2;

use bevy::ecs::event::Event;
use bevy::prelude::EventReader;

use bevy::asset::{AssetServer, Assets};
use bevy::render::render_resource::{Extent3d, TextureFormat};
use bevy::render::texture::Image;

use bevy::prelude::*;

use crate::TerrainMaterialExtension;
use core::fmt::{self, Display, Formatter};

use crate::chunk::{
    save_chunk_collision_data_to_disk, save_chunk_height_map_to_disk, save_chunk_splat_map_to_disk,
    Chunk, ChunkCoordinates, ChunkData, ChunkHeightMapResource,
};
use crate::terrain::{TerrainData, TerrainImageDataLoadStatus};
use crate::terrain_config::TerrainConfig;
use crate::terrain_material::TerrainMaterial;

use bevy_xpbd_3d::prelude::Collider;

use crate::chunk::TerrainChunkMesh;
use anyhow::{Context, Result};

use serde::{Deserialize, Serialize};
use serde_json;

use rand::Rng;

use core::cmp::{max, min};

#[derive(Debug)]
pub enum EditingTool {
    SetHeightMap { height: u16 },        // height, radius, save to disk
    SetSplatMap { r: u8, g: u8, b: u8 }, //R, G, B, radius, save to disk
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub enum BrushType {
    #[default]
    SetExact, // hardness ?
    Smooth,
    Noise,
}

impl Display for BrushType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let label = match self {
            BrushType::SetExact => "SetExact",
            BrushType::Smooth => "Smooth",
            BrushType::Noise => "Noise",
        };

        write!(f, "{}", label)
    }
}

// entity, editToolType, coords, magnitude
#[derive(Event)]
pub struct EditTerrainEvent {
    pub entity: Entity,
    pub tool: EditingTool,
    pub radius: f32,
    pub brush_hardness: f32, //1.0 is full
    pub coordinates: Vec2,
    pub brush_type: BrushType,
}

#[derive(Event)]
pub enum TerrainCommandEvent {
    SaveAllChunks(bool, bool, bool), //height data, splat data, collision data
}

pub fn apply_command_events(
    asset_server: Res<AssetServer>,

    mut chunk_query: Query<(&Chunk, &mut ChunkData, &Parent, &Children)>, //chunks parent should have terrain data

    mut images: ResMut<Assets<Image>>,
    mut terrain_materials: ResMut<Assets<TerrainMaterialExtension>>,

    mut chunk_height_maps: ResMut<ChunkHeightMapResource>,

    terrain_query: Query<(&TerrainData, &TerrainConfig)>,

    chunk_mesh_query: Query<(Entity, &Handle<Mesh>, &GlobalTransform), With<TerrainChunkMesh>>,
    meshes: Res<Assets<Mesh>>,

    mut ev_reader: EventReader<TerrainCommandEvent>,
) {
    for ev in ev_reader.read() {
        for (chunk, chunk_data, parent_terrain_entity, chunk_children) in chunk_query.iter() {
            let terrain_entity_id = parent_terrain_entity.get();

            if terrain_query.get(terrain_entity_id).is_ok() == false {
                continue;
            }

            let (terrain_data, terrain_config) = terrain_query.get(terrain_entity_id).unwrap();

            match ev {
                TerrainCommandEvent::SaveAllChunks(save_height, save_splat, save_collision) => {
                    if *save_height {
                        if let Some(chunk_height_data) =
                            chunk_height_maps.chunk_height_maps.get(&chunk.chunk_id)
                        {
                            save_chunk_height_map_to_disk(
                                chunk_height_data,
                                format!(
                                    "assets/{}/{}.png",
                                    terrain_config.height_folder_path, chunk.chunk_id
                                ),
                            );
                        }
                    }

                    if *save_splat {
                        if let Some(splat_image_handle) = chunk_data.get_splat_texture_image() {
                            if let Some(splat_image) = images.get(splat_image_handle) {
                                save_chunk_splat_map_to_disk(
                                    splat_image,
                                    format!(
                                        "assets/{}/{}.png",
                                        terrain_config.splat_folder_path, chunk.chunk_id
                                    ),
                                );
                            }
                        }
                    }

                    if *save_collision {
                        println!("Generating and saving collision data.. please wait..");
                        for chunk_child in chunk_children {
                            if let Ok((entity, mesh_handle, mesh_transform)) =
                                chunk_mesh_query.get(chunk_child.clone())
                            {
                                let mesh = meshes
                                    .get(mesh_handle)
                                    .expect("No mesh found for terrain chunk");

                                let collider = Collider::trimesh_from_mesh(&mesh)
                                    .expect("Failed to create collider from mesh");

                                let collider_data_serialized =
                                    bincode::serialize(&collider).unwrap();

                                save_chunk_collision_data_to_disk(
                                    collider_data_serialized,
                                    format!(
                                        "assets/{}/{}.col",
                                        terrain_config.collider_data_folder_path, chunk.chunk_id
                                    ),
                                );
                                continue;
                            }

                            println!("saved terrain collider data  ");
                        }
                    }

                    println!("save complete");
                }
            }
        }
    }

    //  Ok(())
}

pub fn apply_tool_edits(
    mut asset_server: Res<AssetServer>,

    mut chunk_query: Query<(Entity, &Chunk, &mut ChunkData, &Parent, &GlobalTransform)>, //chunks parent should have terrain data
    chunk_mesh_query: Query<(&Parent, &GlobalTransform)>,

    mut images: ResMut<Assets<Image>>,
    mut terrain_materials: ResMut<Assets<TerrainMaterialExtension>>,

    mut chunk_height_maps: ResMut<ChunkHeightMapResource>,

    terrain_query: Query<(&TerrainData, &TerrainConfig)>,

    mut ev_reader: EventReader<EditTerrainEvent>,
) {
    for ev in ev_reader.read() {
        eprintln!("-- {:?} -- terrain edit event!", &ev.tool);

        let intersected_entity = &ev.entity;

        //  if let Some((chunk, mut chunk_data)) = chunk_query.get_mut(intersected_entity.clone()).ok()
        if let Some((chunk_entity, _)) = chunk_mesh_query.get(intersected_entity.clone()).ok() {
            let mut chunk_entities_within_range: Vec<Entity> = Vec::new();

            let mut chunk_dimensions = [256, 256]; //compute me from terrain config
            if let Some((_, _, _, terrain_entity, _)) =
                chunk_query.get_mut(chunk_entity.get().clone()).ok()
            {
                if let Some((terrain_data, terrain_config)) =
                    terrain_query.get(terrain_entity.get().clone()).ok()
                {
                    let chunk_rows = terrain_config.chunk_rows;
                    let terrain_dimensions = terrain_config.terrain_dimensions;

                    chunk_dimensions = [
                        terrain_dimensions.x as u32 / chunk_rows,
                        terrain_dimensions.y as u32 / chunk_rows,
                    ];
                }
            }

            //populate chunk_entities_within_range
            for (chunk_entity, _, _, _, chunk_transform) in chunk_query.iter() {
                let tool_coords: &Vec2 = &ev.coordinates;
                let chunk_transform = chunk_transform.translation();
                let chunk_transform_vec2: Vec2 = Vec2::new(chunk_transform.x, chunk_transform.z);

                let chunk_dimensions_vec: Vec2 =
                    Vec2::new(chunk_dimensions.x() as f32, chunk_dimensions.y() as f32);
                let chunk_center_transform =
                    chunk_transform_vec2.add(chunk_dimensions_vec.div(2.0));

                let chunk_local_distance = tool_coords.distance(chunk_center_transform);

                if chunk_local_distance < 800.0 {
                    chunk_entities_within_range.push(chunk_entity);
                }
            }

            //compute average height since we need this for some tools

            let mut total_height: f32 = 0.0;
            let mut heights_len = 0;

            for chunk_entity_within_range in chunk_entities_within_range.clone() {
                if let Some((
                    chunk_entity,
                    chunk,
                    mut chunk_data,
                    terrain_entity,
                    chunk_transform,
                )) = chunk_query.get_mut(chunk_entity_within_range.clone()).ok()
                {
                    if let Some(height_map_data) =
                        &mut chunk_height_maps.chunk_height_maps.get_mut(&chunk.chunk_id)
                    {
                        let tool_coords: &Vec2 = &ev.coordinates;
                        let chunk_transform = chunk_transform.translation();
                        let chunk_transform_vec2: Vec2 =
                            Vec2::new(chunk_transform.x, chunk_transform.z);

                        let tool_coords_local = tool_coords.add(chunk_transform_vec2.neg());

                        //need to make an array of all of the data indices of the terrain that will be set .. hm ?
                        let img_data_length = height_map_data.0.len();

                        //let mut height_changed = false;
                        let radius = &ev.radius;
                        //   let radius_clone = radius.clone();

                        //  let tool_height:f32 = *height as f32;
                        for x in 0..img_data_length {
                            for y in 0..img_data_length {
                                let local_coords = Vec2::new(x as f32, y as f32);
                                if tool_coords_local.distance(local_coords) < *radius {
                                    let original_height = height_map_data.0[x][y];
                                    total_height += original_height as f32;
                                    heights_len += 1;
                                }
                            }
                        }
                    }
                }
            }
            let average_height = total_height as f32 / heights_len as f32;
            // ------
            let radius = &ev.radius;
            let brush_type = &ev.brush_type;

            let brush_hardness = &ev.brush_hardness;
            //apply the tool to each chunk in range
            for chunk_entity_within_range in chunk_entities_within_range {
                if let Some((
                    chunk_entity,
                    chunk,
                    mut chunk_data,
                    terrain_entity,
                    chunk_transform,
                )) = chunk_query.get_mut(chunk_entity_within_range.clone()).ok()
                {
                    //   if let Some(mut terrain_data) = terrain_data_query.get_mut(terrain_entity.get().clone()).ok() { //why cant i find this ?

                    match &ev.tool {
                        EditingTool::SetHeightMap { height } => {
                            if let Some(height_map_data) =
                                &mut chunk_height_maps.chunk_height_maps.get_mut(&chunk.chunk_id)
                            {
                                // if let Some(img) = images.get_mut( height_map_image_handle ){

                                let tool_coords: &Vec2 = &ev.coordinates;
                                let chunk_transform = chunk_transform.translation();
                                let chunk_transform_vec2: Vec2 =
                                    Vec2::new(chunk_transform.x, chunk_transform.z);

                                let tool_coords_local = tool_coords.add(chunk_transform_vec2.neg());

                                //need to make an array of all of the data indices of the terrain that will be set .. hm ?
                                let img_data_length = height_map_data.0.len();

                                let mut height_changed = false;

                                let radius_clone = radius.clone();

                                match brush_type {
                                    BrushType::SetExact => {
                                        for x in 0..img_data_length {
                                            for y in 0..img_data_length {
                                                let local_coords = Vec2::new(x as f32, y as f32);

                                                let hardness_multiplier = get_hardness_multiplier(
                                                    tool_coords_local.distance(local_coords),
                                                    radius_clone,
                                                    *brush_hardness,
                                                );
                                                let original_height = height_map_data.0[x][y];

                                                if tool_coords_local.distance(local_coords)
                                                    < radius_clone
                                                {
                                                    let new_height = height.clone();
                                                    height_map_data.0[x][y] =
                                                        apply_hardness_multiplier(
                                                            original_height as f32,
                                                            new_height as f32,
                                                            hardness_multiplier,
                                                        )
                                                            as u16;
                                                    height_changed = true;
                                                }
                                            }
                                        }
                                    }

                                    BrushType::Smooth => {
                                        for x in 0..img_data_length {
                                            for y in 0..img_data_length {
                                                let local_coords = Vec2::new(x as f32, y as f32);
                                                if tool_coords_local.distance(local_coords)
                                                    < *radius
                                                {
                                                    let hardness_multiplier =
                                                        get_hardness_multiplier(
                                                            tool_coords_local
                                                                .distance(local_coords),
                                                            radius_clone,
                                                            *brush_hardness,
                                                        );

                                                    let original_height = height_map_data.0[x][y];
                                                    // Gather heights of the current point and its neighbors within the brush radius

                                                    let new_height = ((average_height
                                                        + original_height as f32)
                                                        / 2.0)
                                                        as u16;
                                                    height_map_data.0[x][y] =
                                                        apply_hardness_multiplier(
                                                            original_height as f32,
                                                            new_height as f32,
                                                            hardness_multiplier,
                                                        )
                                                            as u16;
                                                    height_changed = true;
                                                }
                                            }
                                        }
                                    }

                                    BrushType::Noise => {
                                        let mut rng = rand::thread_rng();
                                        for x in 0..img_data_length {
                                            for y in 0..img_data_length {
                                                let local_coords = Vec2::new(x as f32, y as f32);
                                                if tool_coords_local.distance(local_coords)
                                                    < *radius
                                                {
                                                    let original_height = height_map_data.0[x][y];
                                                    let hardness_multiplier =
                                                        get_hardness_multiplier(
                                                            tool_coords_local
                                                                .distance(local_coords),
                                                            radius_clone,
                                                            *brush_hardness,
                                                        );

                                                    // Generate a random value between -0.5 and 0.5, then scale it by the desired height variation
                                                    let noise = rng.gen::<f32>() - 0.5;
                                                    let noise_scaled = noise * *height as f32; // Adjust *height to control the scale of the noise
                                                    let new_height = noise_scaled as u16;

                                                    height_map_data.0[x][y] =
                                                        apply_hardness_multiplier(
                                                            original_height as f32,
                                                            new_height as f32,
                                                            hardness_multiplier,
                                                        )
                                                            as u16;
                                                    height_changed = true;
                                                }
                                            }
                                        }
                                    }
                                }

                                if height_changed {
                                    chunk_data.height_map_image_data_load_status =
                                        TerrainImageDataLoadStatus::NeedsReload;
                                }
                            }
                        }

                        EditingTool::SetSplatMap { r, g, b } => {
                            if let Some(splat_image_handle) = chunk_data.get_splat_texture_image() {
                                if let Some(img) = images.get_mut(splat_image_handle) {
                                    // Calculate the pixel position and radius in pixels
                                    let img_size = img.size();

                                    let tool_coords: &Vec2 = &ev.coordinates;
                                    let chunk_transform = chunk_transform.translation();
                                    let chunk_transform_vec2: Vec2 =
                                        Vec2::new(chunk_transform.x, chunk_transform.z);

                                    let chunk_dimensions_vec: Vec2 = Vec2::new(
                                        chunk_dimensions.x() as f32,
                                        chunk_dimensions.y() as f32,
                                    );

                                    let tool_coords_local =
                                        tool_coords.add(chunk_transform_vec2.neg());

                                    let pixel_pos = Vec2::new(
                                        tool_coords_local.x / chunk_dimensions_vec.x
                                            * img_size.x as f32,
                                        tool_coords_local.y / chunk_dimensions_vec.y
                                            * img_size.y as f32,
                                    );
                                    let pixel_radius = *radius as f32;

                                    //force override
                                    //  img.texture_descriptor.format = TextureFormat::Rgba8Unorm;

                                    println!(
                                        "set splat map at {} {} {}",
                                        pixel_pos, pixel_radius, r
                                    );
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
                                                    img.data[idx + 2] = *b as u8;
                                                    // B
                                                    // Alpha value remains unchanged

                                                    //println!("modify pixel data ");
                                                }
                                            }
                                        }

                                        let updated_image = img.clone();

                                        let updated_image_handle = asset_server.add(updated_image);

                                        chunk_data
                                            .set_splat_texture_image(updated_image_handle.clone()); //is this necessary? i think so in case the height is modified

                                        if let Some(material_handle) = &chunk_data.material_handle {
                                            if let Some(terrain_material) =
                                                terrain_materials.get_mut(material_handle)
                                            {
                                                //this should let us avoid rebuilding the entire mesh
                                                terrain_material.extension.splat_texture =
                                                    Some(updated_image_handle);
                                                println!("rewrote splat tex in terrain material ");
                                            }
                                        }

                                    /*
                                    } else if  img.texture_descriptor.format == TextureFormat::Rgba16Unorm
                                      || img.texture_descriptor.format
                                          == TextureFormat::Rgba16Snorm  {




                                               // Iterate over each pixel in the image
                                          for y in 0..img_size.y {
                                              for x in 0..img_size.x {
                                                  let idx = (y * img_size.x + x) as usize * 8; // 8 bytes per pixel (R, G, B, A)
                                                  let pixel_coords = Vec2::new(x as f32, y as f32);


                                                  // Check if the pixel is within the tool's radius
                                                  if pixel_coords.distance(pixel_pos) < pixel_radius {
                                                     // Convert u8 values to u16
                                                      let r_u16 = (*r as u16) * 257; // Equivalent to shifting left by 8 bits and adding the original value for a more accurate representation
                                                      let g_u16 = (*g as u16) * 257;
                                                      let b_u16 = (*b as u16) * 257;

                                                      // Split the u16 into two u8s and store them
                                                      img.data[idx] = (r_u16 & 0xFF) as u8; // R low byte
                                                      img.data[idx + 1] = (r_u16 >> 8) as u8; // R high byte
                                                      img.data[idx + 2] = (g_u16 & 0xFF) as u8; // G low byte
                                                      img.data[idx + 3] = (g_u16 >> 8) as u8; // G high byte
                                                      img.data[idx + 4] = (b_u16 & 0xFF) as u8; // B low byte
                                                      img.data[idx + 5] = (b_u16 >> 8) as u8; // B high byte
                                              }
                                          }
                                      }


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
                                          */

                                    //mark  material as needing reload !!
                                    } else {
                                        println!(
                                            "incorrect splat tex format {:?}",
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
}

fn get_hardness_multiplier(pixel_distance: f32, brush_radius: f32, brush_hardness: f32) -> f32 {
    // Calculate the distance as a percentage of the radius
    let distance_percent = pixel_distance / brush_radius;
    let adjusted_distance_percent = f32::min(1.0, distance_percent); // Ensure it does not exceed 1

    // Calculate the fade effect based on brush hardness
    // When hardness is 0, this will linearly interpolate from 1 at the center to 0 at the edge
    // When hardness is between 0 and 1, it adjusts the fade effect accordingly
    let fade_effect = 1.0 - adjusted_distance_percent;

    // Apply the brush hardness to scale the fade effect, ensuring a minimum of 0
    f32::max(
        0.0,
        fade_effect * (1.0 + brush_hardness) - (adjusted_distance_percent * brush_hardness),
    )
}

fn apply_hardness_multiplier(
    original_height: f32,
    new_height: f32,
    hardness_multiplier: f32,
) -> f32 {
    original_height + (new_height - original_height) * hardness_multiplier
}
