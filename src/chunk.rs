use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use bevy::asset::LoadState;
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::tasks::{AsyncComputeTaskPool, Task};

use bevy::utils::HashMap;
use futures_lite::future;
use image::{GrayImage, ImageBuffer, Luma, RgbaImage};

use crate::heightmap::{HeightMap, HeightMapU16   };
use crate::pre_mesh::PreMesh;
use crate::terrain::{TerrainData, TerrainImageDataLoadStatus, TerrainViewer};
use crate::terrain_config::TerrainConfig;
use crate::terrain_material::{ChunkMaterialUniforms, TerrainMaterial, ToolPreviewUniforms};
use crate::tool_preview::ToolPreviewResource;

use bevy::pbr::ExtendedMaterial;
use bevy::pbr::OpaqueRendererMethod;

use std::fs;

#[derive(Default, Eq, PartialEq)]
enum ChunkState {
    #[default]
    Init,

    Building,
    FullyBuilt,
}

#[derive(Component, Default)]
pub struct Chunk {
    pub chunk_id: u32, //same as chunk index
}

impl Chunk {
    pub fn new(chunk_id: u32) -> Self {
        Self { chunk_id }
    }
}

#[derive(Resource, Default)]
pub struct ChunkHeightMapResource {
    pub chunk_height_maps: HashMap<u32,  HeightMapU16>, // Keyed by chunk id
}

pub type TerrainMaterialExtension = ExtendedMaterial<StandardMaterial, TerrainMaterial>;

#[derive(Component)]
pub struct ChunkData {
    chunk_state: ChunkState,
    lod_level: u8,

    //shouldnt be overly large or else lag
    pub height_map_image_handle: Option<Handle<Image>>,
    pub height_map_image_data_load_status: TerrainImageDataLoadStatus,

    // pub height_map_data: Option<HeightMapU16>,
    splat_image_handle: Option<Handle<Image>>,

   // alpha_mask_image_handle: Option<Handle<Image>>, //built from the height map

    pub material_handle: Option<Handle<TerrainMaterialExtension>>,
}

impl ChunkData {

    pub fn get_height_map_texture_image(&self) -> &Option<Handle<Image>> {
        &self.height_map_image_handle
    }


    pub fn get_splat_texture_image(&self) -> &Option<Handle<Image>> {
        &self.splat_image_handle
    }

    pub fn set_splat_texture_image(&mut self, tex_handle: Handle<Image>) {
        self.splat_image_handle = Some(tex_handle);
    }

  //  pub fn get_alpha_mask_texture_image(&self) -> &Option<Handle<Image>> {
  //      &self.alpha_mask_image_handle
  //  }
}

pub type TerrainPbrBundle = MaterialMeshBundle<TerrainMaterialExtension>;

#[derive(Component)]
pub struct MeshBuilderTask(Task<BuiltChunkMeshData>);

pub struct BuiltChunkMeshData {
    chunk_entity_id: Entity,

    mesh: Mesh,
    chunk_uv: Vec4,

    lod_level: u8,
}

#[derive(Component)]
pub struct TerrainChunkMesh {}

#[derive(Component)]
pub struct CachedHeightmapData {
    pub heightmap_data: Vec<Vec<u16>>,
}

pub trait ChunkCoordinates {
    fn new(x: u32, y: u32) -> Self;

    fn x(&self) -> u32;
    fn y(&self) -> u32;

    fn get_chunk_index(&self, chunk_rows: u32) -> u32;

    fn from_location(
        location: Vec3,
        terrain_origin: Vec3,
        terrain_dimensions: Vec2,
        chunk_rows: u32,
    ) -> Option<UVec2>;
    fn to_location(
        &self,
        terrain_origin: Vec3,
        terrain_dimensions: Vec2,
        chunk_rows: u32,
    ) -> Option<Vec3>;

    fn from_chunk_id(chunk_id: u32, chunk_rows: u32) -> Self;
    fn get_location_offset(&self, chunk_dimensions: Vec2) -> Vec3;

    fn get_heightmap_subsection_bounds_pct(&self, chunk_rows: u32) -> [[f32; 2]; 2];
}

pub type ChunkCoords = [u32; 2];

impl ChunkCoordinates for ChunkCoords {
    fn new(x: u32, y: u32) -> Self {
        [x, y]
    }

    fn x(&self) -> u32 {
        self[0]
    }
    fn y(&self) -> u32 {
        self[1]
    }

    //chunk index is   chunk_col * 64  + chunk_row   IF chunk_rows is 64
    fn get_chunk_index(&self, chunk_rows: u32) -> u32 {
        return self.y() * chunk_rows + self.x() as u32;
    }

    fn from_chunk_id(chunk_id: u32, chunk_rows: u32) -> Self {
        let coords_y = chunk_id / chunk_rows;
        let coords_x = chunk_id % chunk_rows;

        [coords_x, coords_y]
    }

    fn get_location_offset(&self, chunk_dimensions: Vec2) -> Vec3 {
        Vec3::new(
            chunk_dimensions.x * self.x() as f32,
            0.0,
            chunk_dimensions.y * self.y() as f32,
        )
    }

    fn from_location(
        from_location: Vec3,
        terrain_origin: Vec3,
        terrain_dimensions: Vec2,
        chunk_rows: u32,
    ) -> Option<UVec2> {
        let location_delta = from_location - terrain_origin;

        //let terrain_min = terrain_origin;
        //let terrain_max = terrain_origin + Vec3::new(terrain_dimensions.x, 0.0, terrain_dimensions.y);

        // Check if from_location is within the terrain bounds
        if location_delta.x >= 0.0
            && location_delta.x <= terrain_dimensions.x
            && location_delta.z >= 0.0
            && location_delta.z <= terrain_dimensions.y
        {
            // Calculate the chunk's x and z coordinates
            let chunk_x = (location_delta.x / terrain_dimensions.x * chunk_rows as f32) as u32;
            let chunk_z = (location_delta.z / terrain_dimensions.y * chunk_rows as f32) as u32;

            return Some(UVec2::new(chunk_x, chunk_z));
        }

        None
    }

    //returns the middle of the chunk
    fn to_location(
        &self,
        terrain_origin: Vec3,
        terrain_dimensions: Vec2,
        chunk_rows: u32,
    ) -> Option<Vec3> {
        // Ensure chunk coordinates are within bounds
        if self.x() < chunk_rows && self.y() < chunk_rows {
            // Calculate the dimensions of a single chunk
            let chunk_dim_x = terrain_dimensions.x / chunk_rows as f32;
            let chunk_dim_z = terrain_dimensions.y / chunk_rows as f32;

            // Calculate world location for the bottom-left corner of the chunk
            let world_x = terrain_origin.x + self.x() as f32 * chunk_dim_x + chunk_dim_x / 2.0;
            let world_z = terrain_origin.z + self.y() as f32 * chunk_dim_z + chunk_dim_z / 2.0;

            return Some(Vec3::new(world_x, terrain_origin.y, world_z));
        }

        None
    }

    fn get_heightmap_subsection_bounds_pct(&self, chunk_rows: u32) -> [[f32; 2]; 2] {
        let chunk_x = self.x();
        let chunk_y = self.y();

        let pct_per_row = 1.0 / chunk_rows as f32;

        return [
            [chunk_x as f32 * pct_per_row, chunk_y as f32 * pct_per_row], //start corner x and y
            [
                (chunk_x + 1) as f32 * pct_per_row,
                (chunk_y + 1) as f32 * pct_per_row,
            ], //end corner x and y
        ];
    }
}

fn calculate_chunk_coords(
    from_location: Vec3,
    terrain_origin: Vec3,
    terrain_dimensions: Vec2,
    chunk_rows: u32,
) -> [i32; 2] {
    let location_delta = from_location - terrain_origin;

    let chunk_x = (location_delta.x / terrain_dimensions.x * chunk_rows as f32) as i32;
    let chunk_z = (location_delta.z / terrain_dimensions.y * chunk_rows as f32) as i32;

    return [chunk_x, chunk_z];
}

pub fn initialize_chunk_data(
    mut commands: Commands,

    asset_server: Res<AssetServer>,

    mut chunk_query: Query<(Entity, &Chunk, &Parent), Without<ChunkData>>,

    terrain_query: Query<(&TerrainConfig, &TerrainData)>,
) {
    for (chunk_entity, chunk, terrain_entity) in chunk_query.iter_mut() {
        let terrain_entity_id = terrain_entity.get();
        if terrain_query.get(terrain_entity_id).is_ok() == false {
            continue;
        }
        let (terrain_config, terrain_data) = terrain_query.get(terrain_entity_id).unwrap();

        let chunk_id = chunk.chunk_id;
        let file_name = format!("{}.png", chunk_id);

        //default_terrain/diffuse
        let height_texture_path = terrain_config.height_folder_path.join(&file_name);
        println!("loading from {}", height_texture_path.display());

        let height_map_image_handle: Handle<Image> = asset_server.load(height_texture_path);

        //default_terrain/splat
        let splat_texture_path = terrain_config.splat_folder_path.join(&file_name);
        println!("loading from {}", splat_texture_path.display());

        let splat_image_handle: Handle<Image> = asset_server.load(splat_texture_path);


        let chunk_base_lod = 0; // hmm might cause issues .. base this off distance properly ? 
        let lod_level_offset = terrain_config.lod_level_offset;

        let chunk_data_component = ChunkData {
            chunk_state: ChunkState::Init,
            lod_level: chunk_base_lod + lod_level_offset,

            height_map_image_handle: Some(height_map_image_handle),
            //     height_map_data: None, //make this its own component ?
            height_map_image_data_load_status: TerrainImageDataLoadStatus::NotLoaded,

            splat_image_handle: Some(splat_image_handle),
           // alpha_mask_image_handle: None, //gets set later
            material_handle: None,         //gets set later
        };

        commands
            .get_entity(chunk_entity)
            .unwrap()
            .insert(chunk_data_component);
    }
}

/*

Have to do this hack since bevy is not correctly detecting the format

*/

pub fn update_splat_image_formats(
    mut ev_asset: EventReader<AssetEvent<Image>>,
    mut images: ResMut<Assets<Image>>,

    mut chunk_query: Query<(Entity, &Chunk, &ChunkData)>,
) {
    for ev in ev_asset.read() {
        match ev {
            AssetEvent::LoadedWithDependencies { id } => {
                let mut image_is_splat = false;

                let handle = Handle::Weak(*id);

                for (entity, chunk, chunk_data) in chunk_query.iter() {
                    if chunk_data.splat_image_handle == Some(handle.clone()) {
                        image_is_splat = true
                    }
                }

                if image_is_splat {
                    let img = images.get_mut(handle).unwrap();
                    println!("splat image format is {:?}", img.texture_descriptor.format);
                    img.texture_descriptor.format = TextureFormat::Rgba8Unorm;
                }
            }

            _ => {}
        }
    }
}

pub fn reset_chunk_height_data(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,

    chunk_height_maps: Res <ChunkHeightMapResource>,

    mut chunk_query: Query<(Entity, &Chunk, &mut ChunkData, &Parent, &Children)>,
) {
    for (chunk_entity, chunk, mut chunk_data, terrain_entity, children) in chunk_query.iter_mut() {
        if chunk_data.height_map_image_data_load_status == TerrainImageDataLoadStatus::NeedsReload {
            //remove the built mesh
            // for &child in children.iter() {
            //     commands.entity(child).despawn_recursive();
            // }

            chunk_data.chunk_state = ChunkState::Init; // change me ?
                                                       //chunk_data.height_map_image_data_load_status = TerrainImageDataLoadStatus::NotLoaded;

           //  if let Some(height_map_data) = &chunk_height_maps.chunk_height_maps.get(&chunk.chunk_id)
           // {
               // let alpha_mask_image: Image =
              //      build_alpha_mask_image_from_height_data(&height_map_data);
              //  chunk_data.alpha_mask_image_handle = Some(images.add(alpha_mask_image));
           // }

            //we can let go of the height map image handle now that we loaded our heightmap data from it
            //terrain_data.height_map_image_handle = None;
            chunk_data.height_map_image_data_load_status = TerrainImageDataLoadStatus::Loaded;
        }
    }
}

pub fn build_chunk_height_data(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,

    mut chunk_height_maps: ResMut<ChunkHeightMapResource>,

    mut chunk_query: Query<(Entity, &Chunk, &mut ChunkData, &Parent)>,
) {
    for (chunk_entity, chunk, mut chunk_data, terrain_entity) in chunk_query.iter_mut() {
        if chunk_data.height_map_image_data_load_status == TerrainImageDataLoadStatus::NotLoaded {
            let height_map_image: &Image = match &chunk_data.height_map_image_handle {
                Some(height_map_handle) => {
                    let height_map_loaded = asset_server.get_load_state(height_map_handle);

                    if height_map_loaded != Some(LoadState::Loaded) {
                        println!("height map not yet loaded");
                        continue;
                    }

                    images.get(height_map_handle).unwrap()
                }
                None => continue,
            };

            //maybe we can do this in a thread since it is quite cpu intense ?
            let loaded_heightmap_data_result = HeightMapU16::load_from_image(height_map_image);

            println!("built heightmapu16 "); //this is working !! !

            match loaded_heightmap_data_result {
                Ok(loaded_heightmap_data) => {
                    //take out of box

                   // let alpha_mask_image: Image =
                   //     build_alpha_mask_image_from_height_data(&*loaded_heightmap_data);
                    //chunk_data.alpha_mask_image_handle = Some(images.add(alpha_mask_image));

                    chunk_height_maps
                        .chunk_height_maps
                        .insert(chunk.chunk_id,  *loaded_heightmap_data );
                    //    chunk_data.height_map_data = Some(*loaded_heightmap_data);
                }
                Err(e) => {
                    println!("{}", e);
                }
            }

            //we can let go of the height map image handle now that we loaded our heightmap data from it
            //terrain_data.height_map_image_handle = None;
            chunk_data.height_map_image_data_load_status = TerrainImageDataLoadStatus::Loaded;
        }
    }
}

/*
pub fn build_alpha_mask_image_from_height_data(height_map_data: &Vec<Vec<u16>>) -> Image {
    // let width = height_map_image.size().x as usize;
    // let height = height_map_image.size().y as usize;
    let height = height_map_data.len();
    let width = if height > 0 {
        height_map_data[0].len()
    } else {
        0
    };

    const THRESHOLD: u16 = (0.001 * 65535.0) as u16;

    // With the format being R16Uint, each pixel is represented by 2 bytes
    let mut modified_data = Vec::with_capacity(height * width * 4); // 2 bytes per pixelfr R1

    for y in 0..height {
        for x in 0..width {
            //  let index = 2 * (y * width + x); // 2 because of R16Uint
            let height_value = height_map_data[y][x];

            let pixel_value: f32 = if height_value > THRESHOLD { 1.0 } else { 0.0 };
            modified_data.extend_from_slice(&pixel_value.to_le_bytes());
        }
    }

    // Assuming Image has a method from_data for creating an instance from raw data
    let size = Extent3d {
        width: width as u32,
        height: height as u32,
        depth_or_array_layers: 1,
    };
    let dimension = TextureDimension::D2;

    Image::new(
        size, //height_map_image.texture_descriptor.size,
        dimension,
        modified_data,
        TextureFormat::R32Float,
        RenderAssetUsages::default(),
    )
}*/


/*
pub fn build_alpha_mask_image(height_map_image: &Image) -> Image {
    let width = height_map_image.size().x as usize;
    let height = height_map_image.size().y as usize;

    const THRESHOLD: u16 = (0.05 * 65535.0) as u16;

    // With the format being R16Uint, each pixel is represented by 2 bytes
    let mut modified_data = Vec::with_capacity(height * width * 2); // 2 bytes per pixel

    for y in 0..height {
        for x in 0..width {
            let index = 2 * (y * width + x); // 2 because of R16Uint
            let height_value = u16::from_le_bytes([
                height_map_image.data[index],
                height_map_image.data[index + 1],
            ]);

            let pixel_value: f32 = if height_value > THRESHOLD { 1.0 } else { 0.0 };
            modified_data.extend_from_slice(&pixel_value.to_le_bytes());
        }
    }

    // Assuming Image has a method from_data for creating an instance from raw data

    Image::new(
        height_map_image.texture_descriptor.size,
        height_map_image.texture_descriptor.dimension,
        modified_data,
        TextureFormat::R32Float,
         RenderAssetUsages::default()
    )
}
*/
/*
On initialization of terrain entity, the chunk entities should be spawned and they should just remain there forever !!!
 */

//this may lag..
pub fn build_chunk_meshes(
    mut commands: Commands,
    terrain_query: Query<(&TerrainConfig, &TerrainData)>,

    mut chunk_query: Query<(Entity, &Chunk, &mut ChunkData, &Parent, &Visibility)>,

    chunk_height_maps: ResMut<ChunkHeightMapResource>,
    // mut chunk_data_query: Query<( &mut ChunkData )>,
) {
    for (chunk_entity, chunk, mut chunk_data, terrain_entity, visibility) in chunk_query.iter_mut()
    {
        if chunk_data.chunk_state == ChunkState::Init {
            let terrain_entity_id = terrain_entity.get();
            if terrain_query.get(terrain_entity_id).is_ok() == false {
                continue;
            }
            let (terrain_config, terrain_data) = terrain_query.get(terrain_entity_id).unwrap();

            let height_map_data = chunk_height_maps.chunk_height_maps.get(&chunk.chunk_id); // &chunk_data.height_map_data.clone();

            if height_map_data.is_none() {
                println!("chunk is missing height map data .");
                continue;
            }

            if chunk_data.height_map_image_handle.is_none() {
                println!("chunk is missing height map _image_handle .");
                continue;
            }

            if chunk_data.splat_image_handle.is_none() {
                println!("chunk is missing splat_image_handle .");
                continue;
            }

            if visibility == Visibility::Hidden {
                //do not do the intensive calculations to build a chunk mesh until it is 'visible' -- this speeds up initial map loading
                continue;
            }

            println!("build chunk mesh {:?}  ", chunk_entity);

            let thread_pool = AsyncComputeTaskPool::get();

            let chunk_rows = terrain_config.chunk_rows;
            let terrain_dimensions = terrain_config.terrain_dimensions;
            let height_scale = terrain_config.height_scale;

            let height_map_subsection_pct = [[0.0, 0.0], [1.0, 1.0]]; //we use this now since each height map represents its entire chunks topology

            //sample me and build triangle data !!

            // might use lots of RAM ? idk ..
            //maybe we subsection first and THEN build the mesh!  oh well... anyways
            let height_map_data_cloned = ( height_map_data.as_ref().unwrap()).clone();

            let lod_level = chunk_data.lod_level;

            let chunk_uv = Vec4::new(
                //tell the shader how to use the height map for this chunk
                height_map_subsection_pct[0][0],
                height_map_subsection_pct[0][1],
                height_map_subsection_pct[1][0],
                height_map_subsection_pct[1][1],
            );

            let chunk_id_clone = chunk.chunk_id.clone();

            //  let chunk_coords = ChunkCoords::from_chunk_id(chunk_id_clone, chunk_rows);

            let (stitch_data_x_row, stitch_data_y_col) = compute_stitch_data(
                chunk_id_clone,
                chunk_rows,
                terrain_dimensions,
                &chunk_height_maps.chunk_height_maps,
            );

            if stitch_data_x_row.is_none() || stitch_data_y_col.is_none() {
                return;
            }

            //for now, add the unstitched data..
            commands.entity(chunk_entity).insert(CachedHeightmapData {
                heightmap_data: height_map_data_cloned.clone(),
            });

            //these three LOC really take no time at all
            let mut sub_heightmap = (height_map_data_cloned.to_vec());

            stitch_data_x_row.map(|x_row| sub_heightmap.append_x_row(x_row));
            stitch_data_y_col.map(|y_col| sub_heightmap.append_y_col(y_col));

            /*
            commands.entity(chunk_entity).insert(
                CachedHeightmapData {
                    heightmap_data: sub_heightmap.0.clone()
                }
            );  */

            // This is not right for some of the edge chunks -- their

            chunk_data.chunk_state = ChunkState::Building;

            let use_greedy_mesh = terrain_config.use_greedy_mesh;

            let task = thread_pool.spawn(async move {
                println!("trying to build premesh");

                //we add the +1 for stitching data
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

                println!("built premesh   ");

                BuiltChunkMeshData {
                    chunk_entity_id: chunk_entity.clone(),

                    mesh,
                    chunk_uv,
                    lod_level,
                }
            });

            // Spawn new entity and add our new task as a component
            commands.spawn(MeshBuilderTask(task));
        }
    }
}

pub fn finish_chunk_build_tasks(
    mut commands: Commands,
    mut chunk_build_tasks: Query<(Entity, &mut MeshBuilderTask)>, //&Chunk, &mut ChunkData, &Parent,

    mut chunk_query: Query<(Entity, &Chunk, &mut ChunkData, &Parent)>, //&Chunk, &mut ChunkData, &Parent,

    chunk_with_children_query: Query<&Children, With<ChunkData>>,

    mut meshes: ResMut<Assets<Mesh>>,

    terrain_query: Query<(&TerrainData, &TerrainConfig)>,
    mut terrain_materials: ResMut<Assets<TerrainMaterialExtension>>,
) {
    //chunk, mut chunk_data,  terrain_entity,

    for (entity, mut task) in &mut chunk_build_tasks {
        if let Some(built_chunk_mesh_data) = future::block_on(future::poll_once(&mut task.0)) {
            // Add our new PbrBundle of components to our tagged entity

            let chunk_entity_id = built_chunk_mesh_data.chunk_entity_id;

            if chunk_query.get_mut(chunk_entity_id).is_ok() == false {
                continue;
            }
            let (chunk_entity, chunk, mut chunk_data, terrain_entity) =
                chunk_query.get_mut(chunk_entity_id).unwrap();

            let terrain_entity_id = terrain_entity.get();

            let chunk_uv = built_chunk_mesh_data.chunk_uv;
            let mesh = built_chunk_mesh_data.mesh;

            //despawn any old children on this chunk
            if let Ok(chunk_children) = chunk_with_children_query.get(chunk_entity_id) {
                for &child in chunk_children.iter() {
                    commands.entity(child).despawn_recursive();
                }
            }

            //careful w this unwrap
            if terrain_query.get(terrain_entity_id).is_ok() == false {
                continue;
            }

            let (terrain_data, terrain_config) = terrain_query.get(terrain_entity_id).unwrap();
            let color_texture_expansion_factor =  terrain_config.texture_uv_expansion_factor;

            let array_texture = terrain_data.get_array_texture_image().clone();
            let normal_texture = terrain_data.get_normal_texture_image().clone();

            let splat_texture = chunk_data.get_splat_texture_image().clone();
            let height_map_texture = chunk_data.get_height_map_texture_image().clone();

            let chunk_terrain_material: Handle<TerrainMaterialExtension> =
                terrain_materials.add(ExtendedMaterial {
                    base: StandardMaterial {
                        // can be used in forward or deferred mode.
                       // opaque_render_method: OpaqueRendererMethod::Auto,
                       // alpha_mode: AlphaMode::Mask(0.1),

                        reflectance: 0.1,
                        perceptual_roughness: 0.9,
                    //    specular_transmission: 0.1, //kills the depth buffer

                        // in deferred mode, only the PbrInput can be modified (uvs, color and other material properties),
                        // in forward mode, the output can also be modified after lighting is applied.
                        // see the fragment shader `extended_material.wgsl` for more info.
                        // Note: to run in deferred mode, you must also add a `DeferredPrepass` component to the camera and either
                        // change the above to `OpaqueRendererMethod::Deferred` or add the `DefaultOpaqueRendererMethod` resource.
                        ..Default::default()
                    },
                    extension: TerrainMaterial {
                        chunk_uniforms: ChunkMaterialUniforms {
                            color_texture_expansion_factor , //why wont this apply to shader properly ?
                            chunk_uv,
                        },
                        tool_preview_uniforms: ToolPreviewUniforms::default(),
                        diffuse_texture: array_texture.clone(),
                        normal_texture: normal_texture.clone(),
                        splat_texture: splat_texture.clone(),
                        height_map_texture: height_map_texture.clone(),
                        ..default()
                    },
                });

            /*TerrainMaterial {
                base_color_texture:None,
                emissive_texture:None,
                metallic_roughness_texture:None,
                occlusion_texture: None,
                uniforms: ChunkMaterialUniforms {
                    color_texture_expansion_factor: 32.0, //why wont this apply to shader properly ?
                    chunk_uv,
                },

                array_texture: array_texture.clone(),
                splat_texture: splat_texture.clone(),
                alpha_mask_texture: alpha_mask_texture.clone(),
            });*/

            let terrain_mesh_handle = meshes.add(mesh);

            let mesh_bundle = commands
                .spawn(TerrainPbrBundle {
                    mesh: terrain_mesh_handle,
                    material: chunk_terrain_material.clone(),
                    transform: Transform::from_xyz(0.0, 0.0, 0.0),

                    ..default()
                })
                .insert(TerrainChunkMesh {})
                .id();

            chunk_data.material_handle = Some(chunk_terrain_material);

            let mut chunk_entity_commands = commands.get_entity(chunk_entity_id).unwrap();
            chunk_entity_commands.add_child(mesh_bundle);

            chunk_data.chunk_state = ChunkState::FullyBuilt;

            println!("chunk fully built ");

            commands.entity(entity).despawn();
        }
    }
}

pub fn update_tool_uniforms(
    terrain_chunk_mesh_query: Query<&Handle<TerrainMaterialExtension>, With<TerrainChunkMesh>>,

    mut terrain_materials: ResMut<Assets<TerrainMaterialExtension>>,

    tool_preview_resource: Res<ToolPreviewResource>,
) {
    for mat_handle in terrain_chunk_mesh_query.iter() {
        if let Some(mat) = terrain_materials.get_mut(mat_handle) {
            mat.extension.tool_preview_uniforms.tool_coordinates =
                tool_preview_resource.tool_coordinates;
            mat.extension.tool_preview_uniforms.tool_color = tool_preview_resource.tool_color;
            mat.extension.tool_preview_uniforms.tool_radius = tool_preview_resource.tool_radius;
        }
    }
}

pub fn update_chunk_visibility(
    terrain_query: Query<(&TerrainConfig, &TerrainData)>,

    mut chunk_query: Query<(
        &Chunk,
        &mut ChunkData,
        &Parent,
        &GlobalTransform,
        &mut Visibility,
    )>,

    terrain_viewer: Query<&GlobalTransform, With<TerrainViewer>>,
) {
    let viewer = terrain_viewer.get_single();

    let viewer_location: Vec3 = match viewer {
        Ok(view) => view.translation(),
        // FIX: probably should log a warning if there are multiple (or no) viewers, rather than just setting to the origin
        Err(_e) => Vec3::new(0.0, 0.0, 0.0),
    };

    for (chunk, mut chunk_data, parent_entity, chunk_transform, mut chunk_visibility) in
        chunk_query.iter_mut()
    {
        if let Ok((terrain_config, terrain_data)) = terrain_query.get(parent_entity.get()) {
            //  let render_distance_chunks:i32  = terrain_config.get_chunk_render_distance() as i32 ; //make based on render dist
            let lod_level_distance: f32 = terrain_config.get_chunk_lod_distance();
            let lod_level_offset: u8 = terrain_config.lod_level_offset;

            //calc chunk world loc and use to calc the lod
            let chunk_world_location = chunk_transform.translation();

            let distance_to_chunk: f32 = chunk_world_location.distance(viewer_location);

            /*let lod_level: u8 = match distance_to_chunk {
                dist => {
                    if dist > lod_level_distance * 2.0 {
                        2
                    } else if dist > lod_level_distance {
                        1
                    } else {
                        0
                    }
                }
            } + lod_level_offset;*/

            chunk_data.lod_level = lod_level_offset; // for now

            let max_render_distance = terrain_config.get_max_render_distance();

            let should_be_visible = distance_to_chunk <= max_render_distance;

            *chunk_visibility = match should_be_visible {
                true => Visibility::Inherited,
                false => Visibility::Hidden,
            };
        }
    }
}

// outputs as R16 grayscale
/*pub fn save_chunk_height_map_to_disk<P>(
    chunk_height_data: &SubHeightMapU16, // Adjusted for direct Vec<Vec<u16>> input
    save_file_path: P,
) where
    P: AsRef<Path>,
{
    let chunk_height_data = chunk_height_data.0.clone();

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
}*/

pub fn save_chunk_splat_map_to_disk<P>(splat_image: &Image, save_file_path: P)
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
    if format == TextureFormat::Rgba8Unorm || format == TextureFormat::Rgba8UnormSrgb
    //   || format == TextureFormat::Rgba16Unorm
    {
        // The data in Bevy's Image type is stored in a Vec<u8>, so we can use it directly
        let img: RgbaImage = ImageBuffer::from_raw(width, height, image_data.clone())
            .expect("Failed to create image buffer");

        // Save the image to the specified file path
        img.save(&save_file_path).expect("Failed to save splat map");
        println!("saved splat image {}", save_file_path.as_ref().display());
    } else {
        eprintln!("Unsupported image format for saving: {:?}", format);
    }
}

pub fn save_chunk_collision_data_to_disk<P>(serialized_collision_data: Vec<u8>, save_file_path: P)
where
    P: AsRef<Path>,
{
    match fs::write(save_file_path, serialized_collision_data) {
        Ok(_) => println!("Successfully saved collision data to file."),
        Err(e) => println!("Failed to save to file: {}", e),
    }
}

/*

    Attempts to look at adjacent height maps to return stitch data


    THIS IS BUSTED 
*/
pub fn compute_stitch_data(
    chunk_id: u32,
    chunk_rows: u32,
    terrain_dimensions: Vec2,
    chunk_height_maps: &HashMap<u32,  HeightMapU16>,
) -> (Option<Vec<u16>>, Option<Vec<u16>>) {
    let chunk_coords = ChunkCoords::from_chunk_id(chunk_id, chunk_rows);

    let stitch_chunk_id_pos_x =
        ChunkCoords::new(chunk_coords.x() + 1, chunk_coords.y()).get_chunk_index(chunk_rows);
    let stitch_chunk_id_pos_y =
        ChunkCoords::new(chunk_coords.x(), chunk_coords.y() + 1).get_chunk_index(chunk_rows);

    println!(
        "chunk id ... {} {} {} ",
        chunk_id, stitch_chunk_id_pos_x, stitch_chunk_id_pos_y
    );

    let stitch_chunk_id_pos_x_y_corner =
        ChunkCoords::new(chunk_coords.x() + 1, chunk_coords.y() + 1).get_chunk_index(chunk_rows);

    let max_chunk_id_plus_one = chunk_rows * chunk_rows;

    let stitch_data_x_row: Option<Vec<u16>>;

    let stitch_data_y_col: Option<Vec<u16>>;

    let stitch_data_x_y_corner: Option<u16>;

    let chunk_dimensions = [
        terrain_dimensions.x as u32 / chunk_rows,
        terrain_dimensions.y as u32 / chunk_rows,
    ];

    if let Some(chunk_height_data) = chunk_height_maps.get(&stitch_chunk_id_pos_x_y_corner) {
        stitch_data_x_y_corner = Some(chunk_height_data [0][0]);
    } else {
        stitch_data_x_y_corner = Some(0);
    }

// chunk_height_data [y][x]
// this applies  a stitch along the X axis - should pull all values along X axis
    if let Some(chunk_height_data) = chunk_height_maps.get(&stitch_chunk_id_pos_x) {
        let mut final_vec = Vec::new();
        for i in 0..chunk_dimensions.x() as usize {
            final_vec.push(chunk_height_data [i][0]);
        }
        // final_vec.push(stitch_data_x_y_corner.unwrap_or(0)) ;
        stitch_data_x_row = Some(final_vec);
    } else {
        println!("WARN no height data for {:?}", stitch_chunk_id_pos_x);

        if stitch_chunk_id_pos_x < max_chunk_id_plus_one {
            return (None, None);
        }; //prevents loading race cond issue with stitching

        let mut final_vec = Vec::new();
        for _ in 0..chunk_dimensions.x() as usize {
            final_vec.push(0);
        }

        //final_vec.push(stitch_data_x_y_corner.unwrap_or(0)); // the corner corner --gotta fix me some how ?? - try to read diag chunk


        stitch_data_x_row = Some(final_vec);
    }


// this applies  a stitch along the Y axis - should pull all values along y axis
    if let Some(chunk_height_data) = chunk_height_maps.get(&stitch_chunk_id_pos_y) {
        let mut final_vec = Vec::new();
        for i in 0..chunk_dimensions.y() as usize {
            final_vec.push(chunk_height_data [0][i]);
        }
        final_vec.push(stitch_data_x_y_corner.unwrap_or(0)); // the corner corner --gotta fix me some how ?? - try to read diag chunk
        stitch_data_y_col = Some(final_vec);
    } else {
        println!("WARN no height data for {:?}", stitch_chunk_id_pos_y);
        if stitch_chunk_id_pos_y < max_chunk_id_plus_one {
            return (None, None);
        }; //prevents loading race cond issue with stitching

        let mut final_vec = Vec::new();
        for _ in 0..chunk_dimensions.y() as usize {
            final_vec.push(0);
        }
        final_vec.push(stitch_data_x_y_corner.unwrap_or(0)); // the corner corner --gotta fix me some how ?? - try to read diag chunk

        stitch_data_y_col = Some(final_vec);
    }

    (stitch_data_x_row, stitch_data_y_col)
}
