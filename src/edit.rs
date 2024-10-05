use crate::hypersplat::SplatMapDataUpdated;
use crate::hypersplat::save_chunk_splat_index_map_to_disk;
use crate::hypersplat::save_chunk_splat_strength_map_to_disk;
 
use crate::heightmap::HeightMap;
use crate::hypersplat::ChunkSplatDataRaw;
use std::ops::{Add, Div, Neg};
use std::path::PathBuf;

use bevy::ecs::entity::Entity;
use bevy::math::Vec2;

use bevy::ecs::event::Event;
use bevy::prelude::EventReader;

use bevy::asset::{AssetServer, Assets};
use bevy::render::render_resource::{Extent3d, TextureFormat};
use bevy::render::texture::Image;

use bevy::prelude::*;

use crate::pre_mesh::PreMesh;
use crate::TerrainMaterialExtension;
use core::fmt::{self, Display, Formatter};

use crate::chunk::{
    compute_stitch_data, save_chunk_collision_data_to_disk,   
     Chunk, ChunkCoordinates, ChunkCoords, ChunkData,
    ChunkHeightMapResource,
};
use crate::terrain::{TerrainData, TerrainImageDataLoadStatus};
use crate::terrain_config::TerrainConfig;
use crate::terrain_material::TerrainMaterial;
 
use avian3d::prelude::Collider;

 

use crate::chunk::TerrainChunkMesh;
use anyhow::{Context, Result};

use serde::{Deserialize, Serialize};
use serde_json;

use rand::Rng;

use core::cmp::{max, min};

#[derive(Debug, Clone)]
pub enum EditingTool {
    SetHeightMap { height: u16 },        // height, radius, save to disk
    SetSplatMap { r: u8, g: u8, b: u8 }, //R, G, B, radius, save to disk
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub enum BrushType {
    #[default]
    SetExact, // hardness ?
    ClearAll,
    Smooth,
    Noise,
    EyeDropper,
}

impl Display for BrushType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let label = match self {
            BrushType::SetExact => "SetExact",
            BrushType::ClearAll => "ClearAll",
            BrushType::Smooth => "Smooth",
            BrushType::Noise => "Noise",
            BrushType::EyeDropper => "EyeDropper",
        };

        write!(f, "{}", label)
    }
}

// entity, editToolType, coords, magnitude
#[derive(Event, Debug, Clone)]
pub struct EditTerrainEvent {
    pub entity: Entity,
    pub tool: EditingTool,
    pub radius: f32,
    pub brush_hardness: f32, //1.0 is full
    pub coordinates: Vec2,
    pub brush_type: BrushType,
}

#[derive(Event, Debug, Clone)]
pub enum TerrainBrushEvent {
    EyeDropTerrainHeight { height: u16 },
    EyeDropSplatMap { r: u8, g: u8, b: u8 },
}

#[derive(Event, Debug, Clone)]
pub enum TerrainCommandEvent {
    SaveAllChunks(bool, bool, bool), //height data, splat data, collision data
}

pub fn apply_command_events(
    asset_server: Res<AssetServer>,

      chunk_query: Query<(&Chunk, & ChunkData, &ChunkSplatDataRaw, &Parent, &Children)>, //chunks parent should have terrain data

    mut images: ResMut<Assets<Image>>,
    mut terrain_materials: ResMut<Assets<TerrainMaterialExtension>>,

    mut chunk_height_maps: ResMut<ChunkHeightMapResource>,

    terrain_query: Query<(&TerrainData, &TerrainConfig)>,

    chunk_mesh_query: Query<(Entity, &Handle<Mesh>, &GlobalTransform), With<TerrainChunkMesh>>,
    meshes: Res<Assets<Mesh>>,

    mut ev_reader: EventReader<TerrainCommandEvent>,
) {
    for ev in ev_reader.read() {
        for (chunk, chunk_data, chunk_splat_data, parent_terrain_entity, chunk_children) in chunk_query.iter() {
            let terrain_entity_id = parent_terrain_entity.get();

            if terrain_query.get(terrain_entity_id).is_ok() == false {
                continue;
            }

            let (terrain_data, terrain_config) = terrain_query.get(terrain_entity_id).unwrap();

            match ev {
                TerrainCommandEvent::SaveAllChunks(save_height, save_splat, save_collision) => {
                    let file_name = format!("{}.png", chunk.chunk_id);
                    let asset_folder_path = PathBuf::from("assets");
                    if *save_height {
                        if let Some(chunk_height_data) =
                            chunk_height_maps.chunk_height_maps.get(&chunk.chunk_id)
                        {

                            chunk_height_data.save_heightmap_to_image(
                                asset_folder_path
                                    .join(&terrain_config.height_folder_path)
                                    .join(&file_name)
                                    );

                          
                        }
                    }

                    //need to rewrite this !! 
                     if *save_splat {
 
                        let (chunk_splat_index_map_image,chunk_splat_strength_map_image) 
                                = chunk_splat_data.get_images();
                         
                           
                            save_chunk_splat_index_map_to_disk(
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
                                );
                             
                        
                    } 

                    if *save_collision {
                        println!("Generating and saving collision data.. please wait..");
                        for chunk_child in chunk_children {
                            if let Ok((entity, mesh_handle, mesh_transform)) =
                                chunk_mesh_query.get(chunk_child.clone())
                            {
                                /* let mesh = meshes
                                .get(mesh_handle)
                                .expect("No mesh found for terrain chunk");*/


                                let lod_level = terrain_config.collider_lod_level;

                               // let lod_level = 1; // can customize lod level of colliders here
                                let use_greedy_mesh = true;

                                let chunk_rows = terrain_config.chunk_rows;
                                let terrain_dimensions = terrain_config.terrain_dimensions;

                                let height_map_data =
                                    chunk_height_maps.chunk_height_maps.get(&chunk.chunk_id); // &chunk_data.height_map_data.clone();
                                let height_map_data_cloned =
                                    ( height_map_data.as_ref().unwrap()) ;
                                let mut sub_heightmap : Vec<Vec<u16>> =   height_map_data_cloned.to_vec() ;

                                let chunk_id_clone = chunk.chunk_id.clone();

                                let (stitch_data_x_row, stitch_data_y_col) = compute_stitch_data(
                                    chunk_id_clone,
                                    chunk_rows,
                                    terrain_dimensions,
                                    &chunk_height_maps.chunk_height_maps,
                                );

                                if stitch_data_x_row.is_none() || stitch_data_y_col.is_none() {
                                    return;
                                }

                                stitch_data_x_row.map(|x_row| sub_heightmap.append_x_row(x_row));
                                stitch_data_y_col.map(|y_col| sub_heightmap.append_y_col(y_col));

                                let height_scale = terrain_config.height_scale;
                                let sub_texture_dim = [
                                    terrain_dimensions.x / chunk_rows as f32 + 1.0,
                                    terrain_dimensions.y / chunk_rows as f32 + 1.0,
                                ];

                                let mesh = match use_greedy_mesh {
                                    true => PreMesh::from_heightmap_subsection_greedy(
                                        &sub_heightmap,
                                        height_scale,
                                        lod_level,
                                        sub_texture_dim,
                                    ),

                                    false => PreMesh::from_heightmap_subsection(
                                        &sub_heightmap,
                                        height_scale,
                                        lod_level,
                                        sub_texture_dim,
                                    ),
                                }
                                .build();

                                let collider = Collider::trimesh_from_mesh(&mesh)
                                    .expect("Failed to create collider from mesh");

                                let collider_data_serialized =
                                    bincode::serialize(&collider).unwrap();
                                let file_name = format!("{}.col", chunk.chunk_id);
                                save_chunk_collision_data_to_disk(
                                    collider_data_serialized,
                                    asset_folder_path
                                        .join(&terrain_config.collider_data_folder_path)
                                        .join(file_name),
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
    mut commands: Commands, 
    mut asset_server: Res<AssetServer>,

    mut chunk_query: Query<(Entity, &Chunk, &mut ChunkData, &Parent, &GlobalTransform, Option<&mut ChunkSplatDataRaw>)>, //chunks parent should have terrain data
    chunk_mesh_query: Query<(&Parent, &GlobalTransform)>,

    mut images: ResMut<Assets<Image>>,
    mut terrain_materials: ResMut<Assets<TerrainMaterialExtension>>,

    mut chunk_height_maps: ResMut<ChunkHeightMapResource>,

    terrain_query: Query<(&TerrainData, &TerrainConfig)>,

    mut ev_reader: EventReader<EditTerrainEvent>,

    mut evt_writer: EventWriter<TerrainBrushEvent>,
) {
    for ev in ev_reader.read() {
        info!("-- {:?} -- terrain edit event!", &ev.tool);

        let intersected_entity = &ev.entity;

        //  if let Some((chunk, mut chunk_data)) = chunk_query.get_mut(intersected_entity.clone()).ok()
        if let Some((chunk_entity, _)) = chunk_mesh_query.get(intersected_entity.clone()).ok() {
            let mut chunk_entities_within_range: Vec<Entity> = Vec::new();

            let mut chunk_dimensions = [256, 256]; //compute me from terrain config
            if let Some((_, _, _, terrain_entity, _, _ )) =
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
            for (chunk_entity, _, _, _, chunk_transform, _) in chunk_query.iter() {
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
                    mut chunk_splat_data_raw
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
                        let img_data_length = height_map_data.len();

                        //let mut height_changed = false;
                        let radius = &ev.radius;
                        //   let radius_clone = radius.clone();

                        //  let tool_height:f32 = *height as f32;
                        for x in 0..img_data_length {
                            for y in 0..img_data_length {
                                let local_coords = Vec2::new(x as f32, y as f32);
                                if tool_coords_local.distance(local_coords) < *radius {
                                    let original_height = height_map_data[y][x];
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
                    mut chunk_splat_data_raw
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
                                let img_data_length = height_map_data.len();

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
                                                let original_height = height_map_data[y][x];

                                                if tool_coords_local.distance(local_coords)
                                                    < radius_clone
                                                {
                                                    let new_height = height.clone();
                                                    height_map_data[y][x] =
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

                                     BrushType::ClearAll => {
                                        //do nothing 

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

                                                    let original_height = height_map_data[y][x];
                                                    // Gather heights of the current point and its neighbors within the brush radius

                                                    let new_height = ((average_height
                                                        + original_height as f32)
                                                        / 2.0)
                                                        as u16;
                                                    height_map_data[y][x] =
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
                                                    let original_height = height_map_data[y][x];
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

                                                    height_map_data[y][x] =
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

                                    BrushType::EyeDropper => {
                                        // Check if the clicked coordinates are within the current chunk
                                        if tool_coords.x >= chunk_transform_vec2.x
                                            && tool_coords.x
                                                < chunk_transform_vec2.x
                                                    + chunk_dimensions.x() as f32
                                            && tool_coords.y >= chunk_transform_vec2.y
                                            && tool_coords.y
                                                < chunk_transform_vec2.y
                                                    + chunk_dimensions.y() as f32
                                        {
                                            let tool_coords_local =
                                                tool_coords.add(chunk_transform_vec2.neg());
                                            let x = tool_coords_local.x as usize;
                                            let y = tool_coords_local.y as usize;

                                            if x < img_data_length && y < img_data_length {
                                                let local_height = height_map_data[y][x];
                                                evt_writer.send(
                                                    TerrainBrushEvent::EyeDropTerrainHeight {
                                                        height: local_height,
                                                    },
                                                );
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

                          //  todo!("rewrite set splat ");
                            
                            if let Some(  mut chunk_splat_data_raw ) =  chunk_splat_data_raw {
                                //if let Some(img) = images.get_mut(splat_image_handle) {
                                    // Calculate the pixel position and radius in pixels
                                    let splat_dimensions = UVec2::new(
                                        chunk_splat_data_raw.splat_index_map_texture.width() , 
                                        chunk_splat_data_raw.splat_index_map_texture.height() 
                                        ) ;

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
                                            * splat_dimensions.x as f32,
                                        tool_coords_local.y / chunk_dimensions_vec.y
                                            * splat_dimensions.y as f32,
                                    );
                                    let pixel_radius = *radius as f32;

                                    //force override
                                    //  img.texture_descriptor.format = TextureFormat::Rgba8Unorm;

                                    println!(
                                        "set splat map at {} {} {}",
                                        pixel_pos, pixel_radius, r
                                    );



                                            if let Some(mut cmds) = commands.get_entity( chunk_entity ){


                                                cmds.try_insert(SplatMapDataUpdated);



                                            }
                                  

                                    match brush_type {
                                        BrushType::SetExact => {


                                          

                                            // Assuming the image format is Rgba8
                                             
                                                //                let img_data = img.data.as_mut_slice();

                                                // Iterate over each pixel in the image
                                                for y in 0..splat_dimensions.y {
                                                    for x in 0..splat_dimensions.x {
                                                        let idx = (y * splat_dimensions.x + x) as usize * 4; // 4 bytes per pixel (R, G, B, A)
                                                        let pixel_coords =
                                                            Vec2::new(x as f32, y as f32);



                                                      let mut hardness_multiplier =
                                                        get_hardness_multiplier(
                                                            tool_coords_local
                                                                .distance(pixel_coords),
                                                            pixel_radius,
                                                            *brush_hardness,
                                                        );


                                                        //  img.data[idx] = *r as u8;

                                                        // Check if the pixel is within the tool's radius
                                                        if pixel_coords.distance(pixel_pos)
                                                            < pixel_radius
                                                        {

                                                            let texture_type_index = *r as u8;
                                                            let texture_strength = *g as u8; //careful w this on UI ! 

                                                            let texture_layer = *b as u8;  //0 to 3 

                                                            //make hardness_multiplier always be 1.0 if layer 0 ? 
                                                            if texture_layer == 0 {
                                                                hardness_multiplier = 1.0;
                                                            }

                                                            
                                                            let strength_with_hardness =  
                                                                texture_strength as f32 * 
                                                                hardness_multiplier ;
                                                                

                                                                


                                                            
                                                            chunk_splat_data_raw.set_exact_pixel_data(
                                                                x,
                                                                y,
                                                                texture_layer,
                                                                texture_type_index,
                                                                strength_with_hardness as u8 
                                                            );


 
                                                        }
                                                    }
                                                


                                                   
                                            }  
                                        }

                                        BrushType::ClearAll => {
                                            // Assuming the image format is Rgba8
                                             
                                                //                let img_data = img.data.as_mut_slice();

                                                // Iterate over each pixel in the image
                                                for y in 0..splat_dimensions.y {
                                                    for x in 0..splat_dimensions.x {
                                                        let idx = (y * splat_dimensions.x + x) as usize * 4; // 4 bytes per pixel (R, G, B, A)
                                                        let pixel_coords =
                                                            Vec2::new(x as f32, y as f32);

                                                        //  img.data[idx] = *r as u8;

                                                        // Check if the pixel is within the tool's radius
                                                        if pixel_coords.distance(pixel_pos)
                                                            < pixel_radius
                                                        {

                                                           // let texture_type_index = *r as u8;
                                                           // let texture_strength = *g as u8; //careful w this on UI ! 


                                                            chunk_splat_data_raw.clear_all_pixel_data(
                                                                x,
                                                                y 
                                                            );
 
                                                        }
                                                    }
                                                


                                                   
                                            }  
                                        }

                                        BrushType::EyeDropper => {
                                            /* 
                                            if img.texture_descriptor.format
                                                == TextureFormat::Rgba8Unorm
                                                || img.texture_descriptor.format
                                                    == TextureFormat::Rgba8UnormSrgb
                                            {
                                                if tool_coords.x >= chunk_transform_vec2.x
                                                    && tool_coords.x
                                                        < chunk_transform_vec2.x
                                                            + chunk_dimensions.x() as f32
                                                    && tool_coords.y >= chunk_transform_vec2.y
                                                    && tool_coords.y
                                                        < chunk_transform_vec2.y
                                                            + chunk_dimensions.y() as f32
                                                {
                                                    let tool_coords_local =
                                                        tool_coords.add(chunk_transform_vec2.neg());
                                                    let x = tool_coords_local.x as u32;
                                                    let y = tool_coords_local.y as u32;

                                                    if x < img_size.x && y < img_size.y {
                                                        //  let local_height = height_map_data.0[x][y];
                                                        let idx = (y * img_size.x + x) as usize * 4;
                                                        let r = img.data[idx];
                                                        let g = img.data[idx + 1];
                                                        let b = img.data[idx + 2];

                                                        evt_writer.send(
                                                            TerrainBrushEvent::EyeDropSplatMap {
                                                                r,
                                                                g,
                                                                b,
                                                            },
                                                        );
                                                    }
                                                }
                                            } */
                                        }

                                        //brush type
                                        _ => {} //todo !
                                    
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
