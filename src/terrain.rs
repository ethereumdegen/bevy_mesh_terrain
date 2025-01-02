use bevy::asset::{AssetPath, LoadState};
use bevy::image::{ImageFormatSetting, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AddressMode, FilterMode, SamplerDescriptor, TextureDescriptor, TextureFormat,
};
 
use bevy::utils::HashMap;

use crate::chunk::{Chunk, ChunkCoordinates, ChunkCoords, ChunkData, TerrainMaterialExtension};
use crate::heightmap::{HeightMap, HeightMapU16};
use crate::terrain_material::{ChunkMaterialUniforms, TerrainMaterial};

use crate::terrain_config::TerrainConfig;

/*


Chunks should be more persistent

each chunk should have its own heightmap and splat map !!!  these are their own files too.



*/

//attach me to camera
#[derive(Component, Default)]
pub struct TerrainViewer {}

#[derive(Default, PartialEq, Eq)]
pub enum TerrainImageDataLoadStatus {
    //us this for texture image and splat image and alpha mask .. ?
    #[default]
    NotLoaded,
    Loaded,
    NeedsReload,
}

#[derive(Default, PartialEq, Eq)]
pub enum TerrainDataStatus {
    //us this for texture image and splat image and alpha mask .. ?
    #[default]
    NotLoaded,
    Loaded,
}

#[derive(Component, Default)]
pub struct TerrainData {
    // pub chunk_entity_lookup: HashMap<u32,Entity>,  //why is this necessary  ??
    // pub terrain_config: TerrainConfig,
    pub terrain_data_status: TerrainDataStatus,

    texture_image_handle: Option<Handle<Image>>,
    normal_image_handle: Option<Handle<Image>>,
    blend_height_image_handle:Option<Handle<Image>>,

    texture_image_finalized: bool, //need this for now bc of the weird way we have to load an array texture w polling and stuff... GET RID of me ???replace w enum ?
    normal_image_finalized: bool,
    blend_height_image_finalized:bool ,
}

impl TerrainData {
    pub fn new() -> Self {
        let terrain_data = TerrainData::default();

        //  terrain_data.texture_image_handle = Some(handle.clone()); //strong clone

        terrain_data
    }
}

pub fn initialize_terrain(
    mut commands: Commands,
    mut terrain_query: Query<(Entity, &mut TerrainData, &TerrainConfig)>,
) {
    for (terrain_entity, mut terrain_data, terrain_config) in terrain_query.iter_mut() {
        if terrain_data.terrain_data_status == TerrainDataStatus::NotLoaded {
            let max_chunks = terrain_config.chunk_rows * terrain_config.chunk_rows;

            for chunk_id in 0..max_chunks {
                let chunk_coords = ChunkCoords::from_chunk_id(chunk_id, terrain_config.chunk_rows); // [ chunk_id / terrain_config.chunk_rows ,  chunk_id  % terrain_config.chunk_rows];
                let chunk_dimensions = terrain_config.get_chunk_dimensions();

                let chunk_name = format!("Chunk {:?}", chunk_id);

                let chunk_entity = commands
                    .spawn(Chunk::new(chunk_id))
                    .insert(Name::new(chunk_name))
                    .insert(

                        (

                            Transform::from_xyz(
                            chunk_coords.x() as f32 * chunk_dimensions.x,
                            0.0,
                            chunk_coords.y() as f32 * chunk_dimensions.y,
                            ),
                             Visibility::Hidden
                        )



                       )
                    .id();

                let mut terrain_entity_commands = commands.get_entity(terrain_entity).unwrap();

                //terrain_data.chunk_entity_lookup.insert(chunk_id,chunk_entity.clone());
                terrain_entity_commands.add_child(chunk_entity);
            }

            terrain_data.terrain_data_status = TerrainDataStatus::Loaded
        }
    }
}

impl TerrainData {
    pub fn get_array_texture_image(&self) -> &Option<Handle<Image>> {
        &self.texture_image_handle
    }

    pub fn get_normal_texture_image(&self) -> &Option<Handle<Image>> {
        &self.normal_image_handle
    }

      pub fn get_blend_height_texture_image(&self) -> &Option<Handle<Image>> {
        &self.blend_height_image_handle
    }
    /*


    pub fn add_array_texture_image(mut self, handle: Handle<Image>, sections: u32  )-> Self{
        self.texture_image_handle = Some(handle.clone()); //strong clone
        self.texture_image_sections = sections;
        self
    }


     */
}

//consider building a custom loader for this , not  Image
pub fn load_terrain_texture_from_image(
    mut terrain_query: Query<(&mut TerrainData, &TerrainConfig)>,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    //  materials: Res <Assets<TerrainMaterialExtension>>,
) {
    for (mut terrain_data, terrain_config) in terrain_query.iter_mut() {
        if terrain_data.texture_image_handle.is_none() {
            let array_texture_path = &terrain_config.diffuse_folder_path;




            let tex_image : Handle<Image> = asset_server.load_with_settings(
                AssetPath::from_path(array_texture_path),  |s: &mut ImageLoaderSettings| 
                 {

                    s.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                        label: None,
                        address_mode_u: AddressMode::Repeat.into(),
                        address_mode_v: AddressMode::Repeat.into(),
                        address_mode_w: AddressMode::Repeat.into(),
                        mag_filter: FilterMode::Linear.into(),
                        min_filter: FilterMode::Linear.into(),
                        mipmap_filter: FilterMode::Linear.into(),
                        ..default()
                    });
                 }
                );



           // let tex_image = asset_server.load(AssetPath::from_path(array_texture_path));
            terrain_data.texture_image_handle = Some(tex_image);
        }

        //try to load the height map data from the height_map_image_handle
        if !terrain_data.texture_image_finalized {
            let texture_image: &mut Image = match &terrain_data.texture_image_handle {
                Some(texture_image_handle) => {
                    let texture_image_loaded = asset_server.get_load_state(texture_image_handle);

                    if texture_image_loaded .is_some_and(|st|   st.is_loaded() ) {
                        
                        images.get_mut(texture_image_handle).unwrap()
                    }else {
                        continue ;
                    }
 
                }
                None => continue,
            };

            //https://github.com/bevyengine/bevy/pull/10254
           /* texture_image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                label: None,
                address_mode_u: AddressMode::Repeat.into(),
                address_mode_v: AddressMode::Repeat.into(),
                address_mode_w: AddressMode::Repeat.into(),
                mag_filter: FilterMode::Linear.into(),
                min_filter: FilterMode::Linear.into(),
                mipmap_filter: FilterMode::Linear.into(),
                ..default()
            });*/

            // Create a new array texture asset from the loaded texture.
            let desired_array_layers = terrain_config.texture_image_sections;

            let need_to_reinterpret = desired_array_layers > 1
                && texture_image.texture_descriptor.size.depth_or_array_layers == 1;

            if need_to_reinterpret {
                //info!("texture info {:?}" , texture_image.texture_descriptor.dimension, texture_image.size().depth_or_array_layers);

                texture_image.reinterpret_stacked_2d_as_array(desired_array_layers);
            }
 

            terrain_data.texture_image_finalized = true;
        }
    }
}

pub fn load_terrain_normal_from_image(
    mut terrain_query: Query<(&mut TerrainData, &TerrainConfig)>,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    //  materials: Res <Assets<TerrainMaterialExtension>>,
) {
    for (mut terrain_data, terrain_config) in terrain_query.iter_mut() {
        if terrain_data.normal_image_handle.is_none() {
            let normal_texture_path = &terrain_config.normal_folder_path;


            let tex_image : Handle<Image> = asset_server.load_with_settings(
                AssetPath::from_path(normal_texture_path),  |s: &mut ImageLoaderSettings| 
                 {

                    s.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                        label: None,
                        address_mode_u: AddressMode::Repeat.into(),
                        address_mode_v: AddressMode::Repeat.into(),
                        address_mode_w: AddressMode::Repeat.into(),
                        mag_filter: FilterMode::Linear.into(),
                        min_filter: FilterMode::Linear.into(),
                        mipmap_filter: FilterMode::Linear.into(),
                        ..default()
                    });
                 }
                );

            // let tex_image = asset_server.load(AssetPath::from_path(normal_texture_path));
            terrain_data.normal_image_handle = Some(tex_image);
        }

        
        if !terrain_data.normal_image_finalized {
            let texture_image: &mut Image = match &terrain_data.normal_image_handle {
                Some(texture_image_handle) => {
                    let texture_image_loaded = asset_server.get_load_state(texture_image_handle);

                    if texture_image_loaded.is_some_and(|st|  st.is_loaded() ) {
                        images.get_mut(texture_image_handle).unwrap()
                    }else {
                        continue ;
                    }

                   
                }
                None => continue,
            };

            //https://github.com/bevyengine/bevy/pull/10254
            /*texture_image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                label: None,
                address_mode_u: AddressMode::Repeat.into(),
                address_mode_v: AddressMode::Repeat.into(),
                address_mode_w: AddressMode::Repeat.into(),
                mag_filter: FilterMode::Linear.into(),
                min_filter: FilterMode::Linear.into(),
                mipmap_filter: FilterMode::Linear.into(),
                ..default()
            });*/

            // Create a new array texture asset from the loaded texture.
            let desired_array_layers = terrain_config.texture_image_sections;

            let need_to_reinterpret = desired_array_layers > 1
                && texture_image.texture_descriptor.size.depth_or_array_layers == 1;

            if need_to_reinterpret {
                //info!("texture info {:?}" , texture_image.texture_descriptor.dimension, texture_image.size().depth_or_array_layers);

                texture_image.reinterpret_stacked_2d_as_array(desired_array_layers);
            }

           

            terrain_data.normal_image_finalized = true;
        }
    }
}




pub fn load_terrain_blend_height_from_image(
    mut terrain_query: Query<(&mut TerrainData, &TerrainConfig)>,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    //  materials: Res <Assets<TerrainMaterialExtension>>,
) {
    for (mut terrain_data, terrain_config) in terrain_query.iter_mut() {
        if terrain_data.blend_height_image_handle.is_none() {
            let blend_height_texture_path = &terrain_config.blend_height_folder_path;

        //    let tex_image = asset_server.load(AssetPath::from_path(blend_height_texture_path));


            let tex_image : Handle<Image> = asset_server.load_with_settings(
                AssetPath::from_path(blend_height_texture_path),  |s: &mut ImageLoaderSettings| 
                 {
                    
                    s.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                        label: None,
                        address_mode_u: AddressMode::Repeat.into(),
                        address_mode_v: AddressMode::Repeat.into(),
                        address_mode_w: AddressMode::Repeat.into(),
                        mag_filter: FilterMode::Linear.into(),
                        min_filter: FilterMode::Linear.into(),
                        mipmap_filter: FilterMode::Linear.into(),
                        ..default()
                    });
                 }
                );


            terrain_data.blend_height_image_handle = Some(tex_image);
        }

        
        if !terrain_data.blend_height_image_finalized {
            let texture_image: &mut Image = match &terrain_data.blend_height_image_handle {
                Some(texture_image_handle) => {
                    let texture_image_loaded = asset_server.get_load_state(texture_image_handle);

                    if texture_image_loaded.is_some_and(|st|  st.is_loaded() ) {
                        images.get_mut(texture_image_handle).unwrap()
                    }else {
                        continue ;
                    }

                   
                }
                None => continue,
            };


            let format = texture_image.texture_descriptor.format;
            //force format 
            //texture_image.texture_descriptor.format = TextureFormat::R16Uint ;

          //  if format != TextureFormat::R16Uint {
          //      panic!("blend heightmap: wrong format {:?}", format);
             
           // }
     


            //https://github.com/bevyengine/bevy/pull/10254
           /* texture_image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                label: None,
                address_mode_u: AddressMode::Repeat.into(),
                address_mode_v: AddressMode::Repeat.into(),
                address_mode_w: AddressMode::Repeat.into(),
                mag_filter: FilterMode::Linear.into(),  //affects binding filter mode 
                min_filter: FilterMode::Linear.into(),
                mipmap_filter: FilterMode::Linear.into(),
                ..default()
            });*/

            // Create a new array texture asset from the loaded texture.
            let desired_array_layers = terrain_config.texture_image_sections;

            let need_to_reinterpret = desired_array_layers > 1
                && texture_image.texture_descriptor.size.depth_or_array_layers == 1;

            if need_to_reinterpret {
                //info!("texture info {:?}" , texture_image.texture_descriptor.dimension, texture_image.size().depth_or_array_layers);

                texture_image. reinterpret_stacked_2d_as_array(desired_array_layers);
            }

           

            terrain_data.blend_height_image_finalized = true;
        }
    }
}

